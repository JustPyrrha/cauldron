#![feature(fn_traits)]
#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]

mod core_ui;

pub mod metadata;
pub mod version;

use crate::metadata::{ContributorsList, PluginMetadataSchemaVersionOnly, PluginMetadataV0};
use ::log::LevelFilter;
use egui::TextBuffer;
use serde_derive::Deserialize;
use simplelog::{ColorChoice, Config, TerminalMode};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::fs::{DirEntry, File};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use toml::Value;
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

pub struct CauldronLoader {
    // <id, plugin>
    pub plugins: HashMap<String, PluginContainer>,
}

impl CauldronLoader {
    pub fn new() -> Self {
        CauldronLoader {
            plugins: HashMap::new(),
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
            // when added new plugin meta versions, do migrations from old to new here.
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
        self.plugins.insert(
            metadata.cauldron.id.clone(),
            PluginContainer {
                plugin,
                handle,
                metadata,
            },
        );
        // todo: load order sorting
    }

    fn do_plugin_init(&self) {
        let mut table = tabled::builder::Builder::new();
        table.push_record(["Order", "Id", "Version", "Name", "Description", "Authors"]);
        self.plugins
            .iter()
            .enumerate()
            .for_each(|(index, (id, plugin))| {
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
                    format!("{}", id),
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
        self.plugins.iter().for_each(|(_, plugin)| {
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
    unsafe {
        AllocConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("cauldron/cauldron.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("cauldron: starting v{}...", env!("CARGO_PKG_VERSION"));

    INSTANCE.get_or_init(|| {
        let mut instance = CauldronLoader::new();
        let paths = instance.try_find_plugins();
        for path in paths {
            instance.try_load_plugin(&path);
        }
        instance.do_plugin_init();

        instance
    });

    focus::util::enable_debug_interface(false);
    focus::Focus::builder()
        .with::<focus::hooks::dx12::Dx12Hooks>(core_ui::CauldronUI::new(
            INSTANCE
                .get()
                .unwrap()
                .plugins
                .iter()
                .map(|v| v.1.metadata.clone())
                .collect::<Vec<_>>(),
        ))
        .build()
        .apply()
        .unwrap();
}
