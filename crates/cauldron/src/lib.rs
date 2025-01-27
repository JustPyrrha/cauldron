#![feature(fn_traits)]
#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]

mod core_ui;

pub mod config;
pub mod metadata;
pub mod util;
pub mod version;

use crate::config::load_config;
use crate::metadata::{
    ContributorsList, PluginMetadataDependency, PluginMetadataSchemaVersionOnly, PluginMetadataV0,
};
use crate::util::message_box;
use crate::version::{CauldronGameType, GameVersion};
use egui::Key;
use semver::{Version, VersionReq};
use simplelog::{ColorChoice, Config, SharedLogger, TerminalMode};
use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env::{current_dir, current_exe};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK};
use windows_sys::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};

pub trait CauldronPlugin {
    fn new() -> Self
    where
        Self: Sized;
    fn on_init(&self, _loader: &CauldronLoader) {}
    fn on_deinit(&self) {}
}

pub type PluginBox = Box<dyn CauldronPlugin + Send + Sync>;

pub struct PluginContainer {
    pub plugin: PluginBox,
    pub handle: libloading::Library,
    pub metadata: PluginMetadataV0,
}

pub struct GameInfo {
    pub game_type: CauldronGameType,
    pub version: GameVersion,
}

pub struct CauldronLoader {
    pub plugins: Vec<PluginContainer>,
    pub game: GameInfo,
}

impl CauldronLoader {
    pub fn new() -> Self {
        CauldronLoader {
            plugins: Vec::new(),
            game: GameInfo {
                game_type: CauldronGameType::find_from_exe().unwrap(),
                version: version::version(),
            },
        }
    }

    unsafe fn try_find_plugins(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let dir = current_dir().expect("CauldronLoader::try_find_plugins current_dir failed");
        let dir = dir.join("cauldron").join("plugins");

        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }

        //todo: theres a better way to do this, without duplicating code.
        for entry in fs::read_dir(dir)
            .expect("CauldronLoader::try_find_plugins failed to read plugins directory")
        {
            let Ok(entry) = entry else {
                continue;
            };
            let entry = entry.path();
            if entry.is_dir() {
                for entry in fs::read_dir(entry)
                    .expect("CauldronLoader::try_find_plugins failed to plugin read directory")
                {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let entry = entry.path();
                    if entry.to_str().unwrap().ends_with(".dll") {
                        paths.push(entry);
                    }
                }
            } else if entry.to_str().unwrap().ends_with(".dll") {
                paths.push(entry);
            }
        }

        paths
    }

    unsafe fn try_load_plugin(&mut self, plugin_path: &PathBuf) {
        let handle = libloading::Library::new(&plugin_path);
        let Ok(handle) = handle else {
            error!("cauldron: failed to load plugin: {}", plugin_path.display());
            return;
        };
        let metadata =
            handle.get::<extern "C" fn() -> &'static str>(b"__cauldron_plugin__metadata\0");
        let Ok(metadata) = metadata else {
            error!(
                "cauldron: not a valid plugin (missing metadata export): {}",
                plugin_path.display()
            );
            return;
        };
        let plugin = handle.get::<extern "C" fn() -> PluginBox>(b"__cauldron_plugin__new\0");
        let Ok(plugin) = plugin else {
            error!(
                "cauldron: not a valid plugin (missing create instance export): {}",
                plugin_path.display()
            );
            return;
        };
        let metadata_str = metadata();
        let metadata = toml::from_str::<PluginMetadataSchemaVersionOnly>(metadata_str);
        let Ok(metadata) = metadata else {
            error!(
                "cauldron: failed to parse plugin metadata: {}",
                plugin_path.display()
            );
            return;
        };
        let metadata = match metadata.schema_version {
            0 => toml::from_str::<PluginMetadataV0>(metadata_str),
            // when adding new plugin meta versions, do migrations from old to new here.
            _ => {
                return;
            }
        };
        let Ok(metadata) = metadata else {
            error!(
                "cauldron: failed to parse plugin metadata: {}",
                plugin_path.display()
            );
            return;
        };
        let plugin = plugin();
        self.plugins.push(PluginContainer {
            plugin,
            handle,
            metadata,
        });

        self.sort_and_validate_plugins();
    }

    fn sort_and_validate_plugins(&mut self) {
        self.plugins.sort_by(|a, b| {
            let a_id = &a.metadata.cauldron.id.clone();
            let b_id = &b.metadata.cauldron.id.clone();
            let a_deps = a.metadata.cauldron.dependencies.clone();
            let b_deps = b.metadata.cauldron.dependencies.clone();

            if a_deps.as_ref().is_some_and(|a| a.contains_key(b_id))
                && b_deps.as_ref().is_some_and(|b| b.contains_key(a_id))
            {
                error!(
                    "cauldron: circular dependencies detected. ({} and {} depend on each other.)",
                    a_id, b_id
                );
                message_box(
                    "cauldron: plugin error",
                    format!(
                        "Circular dependencies detected.\n{} and {} depend on each other.",
                        a_id, b_id
                    )
                    .as_str(),
                    MB_OK | MB_ICONERROR,
                );
                std::process::exit(0);
            } else if a_deps.as_ref().is_some_and(|a| a.contains_key(b_id)) {
                Ordering::Greater
            } else if b_deps.as_ref().is_some_and(|b| b.contains_key(a_id)) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

        let mut ids = self
            .plugins
            .iter()
            .map(|p| p.metadata.cauldron.id.clone())
            .collect::<Vec<_>>();
        ids.push(self.game.game_type.id().clone());
        let mut versions: HashMap<String, Version> = HashMap::new();
        self.plugins.iter().for_each(|p| {
            let Ok(version) = Version::parse(p.metadata.cauldron.version.clone().as_str()) else {
                error!(
                    "cauldron: {}'s version ({}) does not match semver requirements. exiting...",
                    p.metadata.cauldron.id, p.metadata.cauldron.version
                );
                message_box(
                    "cauldron: plugin error",
                    format!(
                        "{}'s version ({}) does not match semver requirements.",
                        p.metadata.cauldron.id, p.metadata.cauldron.version
                    )
                    .as_str(),
                    MB_OK | MB_ICONERROR,
                );
                std::process::exit(0);
            };
            versions.insert(p.metadata.cauldron.id.clone(), version);
        });

        for plugin in &self.plugins {
            if plugin.metadata.cauldron.dependencies.is_some() {
                for (dep, constraints) in &plugin.metadata.cauldron.dependencies.clone().unwrap() {
                    if dep.as_str() == self.game.game_type.id().as_str() {
                        // todo: validate version requirements for game version
                        continue;
                    }
                    match constraints {
                        PluginMetadataDependency::Plain(version) => {
                            let Ok(version_req) = VersionReq::parse(version.as_str()) else {
                                error!("cauldron: malformed dependency version requirement constraint {} in {}", version, plugin.metadata.cauldron.id);
                                message_box("cauldron: plugin error", format!("malformed dependency version requirement constraint {} in {}.", version, plugin.metadata.cauldron.id).as_str(), MB_OK | MB_ICONERROR);
                                std::process::exit(0);
                            };

                            if versions.contains_key(dep) {
                                if !version_req.matches(&versions[dep]) {
                                    error!(
                                        "{} {} {}",
                                        version_req.to_string(),
                                        &versions[dep].to_string(),
                                        dep
                                    );

                                    error!("cauldron: plugin {} is missing dependency {} {} (version mismatch)", plugin.metadata.cauldron.id, dep, version);
                                    message_box("cauldron: plugin error", format!("plugin {} is missing dependency {} {} (version mismatch)", plugin.metadata.cauldron.id, dep, version).as_str(), MB_OK | MB_ICONERROR);
                                    std::process::exit(0);
                                }
                            } else {
                                error!(
                                    "cauldron: plugin {} is missing dependency {} {}",
                                    plugin.metadata.cauldron.id, dep, version
                                );
                                message_box(
                                    "cauldron: plugin error",
                                    format!(
                                        "plugin {} is missing dependency {} {}",
                                        plugin.metadata.cauldron.id, dep, version
                                    )
                                    .as_str(),
                                    MB_OK | MB_ICONERROR,
                                );
                                std::process::exit(0);
                            }
                        }
                        PluginMetadataDependency::Detailed(detailed) => {
                            let Ok(version_req) = VersionReq::parse(detailed.version.as_str())
                            else {
                                error!("cauldron: malformed dependency version requirement constraint {} in {}", detailed.version, plugin.metadata.cauldron.id);
                                message_box("cauldron: plugin error", format!("malformed dependency version requirement constraint {} in {}.", detailed.version, plugin.metadata.cauldron.id).as_str(), MB_OK | MB_ICONERROR);
                                std::process::exit(0);
                            };

                            if versions.contains_key(dep) {
                                if !version_req.matches(&versions[dep]) {
                                    error!("cauldron: plugin {} is missing dependency {} {} (version mismatch)", plugin.metadata.cauldron.id, dep, detailed.version);
                                    message_box("cauldron: plugin error", format!("plugin {} is missing dependency {} {} (version mismatch)", plugin.metadata.cauldron.id, dep, detailed.version).as_str(), MB_OK | MB_ICONERROR);
                                    std::process::exit(0);
                                }
                            } else if !detailed.optional {
                                error!(
                                    "cauldron: plugin {} is missing dependency {} ({})",
                                    plugin.metadata.cauldron.id, dep, detailed.version
                                );
                                message_box(
                                    "cauldron: plugin error",
                                    format!(
                                        "plugin {} is missing dependency {} {}",
                                        plugin.metadata.cauldron.id, dep, detailed.version
                                    )
                                    .as_str(),
                                    MB_OK | MB_ICONERROR,
                                );
                                std::process::exit(0);
                            }
                        }
                    }
                }
            }
        }
    }

    fn do_plugin_init(&self) {
        let mut table = tabled::builder::Builder::new();
        table.push_record(["Order", "Id", "Version", "Name", "Description", "Authors"]);
        self.plugins.iter().enumerate().for_each(|(index, plugin)| {
            let mut name = String::new();
            let mut description = String::new();
            let mut authors = String::new();

            if plugin.metadata.cauldron.metadata.is_some() {
                let meta = plugin.metadata.cauldron.metadata.as_ref().unwrap();
                name = meta.name.clone().unwrap_or(String::new());
                description = meta.description.clone().unwrap_or(String::new());
                if meta.contributors.is_some() {
                    let contributors = meta.contributors.as_ref().unwrap();
                    match contributors {
                        ContributorsList::Plain(contributors) => {
                            authors = contributors.join(", ");
                        }
                        ContributorsList::WithRoles(contributors) => {
                            authors = contributors
                                .keys()
                                .map(|a| a.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                        }
                    }
                }
            }

            table.push_record([
                format!("{}", index),
                format!("{}", &plugin.metadata.cauldron.id),
                format!("{}", &plugin.metadata.cauldron.version),
                format!("{}", name),
                format!("{}", description),
                format!("{}", authors),
            ]);
        });
        info!(
            "cauldron: found {} plugins:\n{}",
            self.plugins.len(),
            table.build()
        );

        info!("cauldron: initializing plugins...");
        self.plugins.iter().for_each(|plugin| {
            plugin.plugin.on_init(&self);
        });
        info!("cauldron: plugins initialized.");
    }
}

#[macro_export]
macro_rules! define_cauldron_plugin {
    ($plugin:ty, $meta:expr) => {
        #[cfg(not(test))]
        mod __cauldron_plugin {
            use super::*;

            #[no_mangle]
            extern "C" fn __cauldron_plugin__metadata() -> &'static str {
                $meta
            }

            #[no_mangle]
            extern "C" fn __cauldron_plugin__new() -> $crate::PluginBox {
                Box::new(<$plugin as $crate::CauldronPlugin>::new())
            }
        }
    };
}

pub mod log {
    #[macro_export]
    macro_rules! trace {
        ($($arg:tt)+) => (::log::log!(::log::Level::Trace, $($arg)+))
    }

    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)+) => (::log::log!(::log::Level::Debug, $($arg)+))
    }

    #[macro_export]
    macro_rules! error {
        ($($arg:tt)+) => (::log::log!(::log::Level::Error, $($arg)+))
    }

    #[macro_export]
    macro_rules! warn {
        ($($arg:tt)+) => (::log::log!(::log::Level::Warn, $($arg)+))
    }

    #[macro_export]
    macro_rules! info {
        ($($arg:tt)+) => (::log::log!(::log::Level::Info, $($arg)+))
    }
}

#[doc(hidden)]
static mut INSTANCE: OnceCell<CauldronLoader> = OnceCell::new();

#[doc(hidden)]
pub unsafe fn handle_dll_attach() {
    let config = load_config();
    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    if config.logging.show_console {
        unsafe {
            AllocConsole();
            AttachConsole(ATTACH_PARENT_PROCESS);
        }

        loggers.push(simplelog::TermLogger::new(
            config.logging.console_level.to_log(),
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ))
    }
    loggers.push(simplelog::WriteLogger::new(
        config.logging.file_level.to_log(),
        Config::default(),
        File::create(config.logging.file_path).unwrap(),
    ));
    simplelog::CombinedLogger::init(loggers).unwrap();

    info!("cauldron: starting v{}...", env!("CARGO_PKG_VERSION"));

    if CauldronGameType::find_from_exe().is_none() {
        error!(
            "cauldron: Unknown game type \"{}\", exiting.",
            current_exe()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
        message_box(
            "Game Unknown",
            "Cauldron as detected an unknown game type and will now exit.",
            MB_OK | MB_ICONERROR,
        );
        std::process::exit(0);
    }

    #[allow(static_mut_refs)]
    INSTANCE.get_or_init(|| {
        let mut instance = CauldronLoader::new();
        let paths = instance.try_find_plugins();
        for path in paths {
            instance.try_load_plugin(&path);
        }
        instance.do_plugin_init();

        instance
    });

    if config.ui.enabled {
        if config.ui.enable_dx12_debug {
            focus::util::enable_debug_interface(config.ui.enable_dx12_debug_gpu_validation);
        }
        focus::Focus::builder()
            .with::<focus::hooks::dx12::Dx12Hooks>(core_ui::CauldronUI::new(
                INSTANCE
                    .get()
                    .unwrap()
                    .plugins
                    .iter()
                    .map(|p| p.metadata.clone())
                    .collect::<Vec<_>>(),
                Key::from_name(config.ui.key.as_str()).expect("cauldron: failed to parse ui key"),
            ))
            .build()
            .apply()
            .unwrap();
    }
}
