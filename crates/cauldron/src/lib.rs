#![feature(fn_traits)]
#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]

pub mod loader;
pub mod prelude;
pub mod version;
mod core_plugin;

use crate::loader::on_dll_attach;
use crate::version::{GameType, RuntimeVersion};
use semver::{Version, VersionReq};
use std::sync::OnceLock;
use std::thread;

#[derive(Debug, Clone)]
pub struct PluginDependency {
    /// The ID of the plugin to depend on.
    pub id: String,
    /// The version requirements of the dependency.
    pub versions: VersionReq,
}

impl PluginDependency {
    /// Creates a new plugin dependency spec.
    pub fn new(id: impl Into<String>, versions: VersionReq) -> Self {
        Self {
            id: id.into(),
            versions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginMeta {
    // todo: theres a lot of plugin meta and it gets kinda messy, maybe replace with a toml file?
    /// The plugin's ID.
    /// MUST match ^[a-z][a-z0-9-_]{1,63}$
    pub id: String,

    /// The plugin's SemVer compliant version.
    pub version: Version,

    /// The Decima game the plugin is compatible with.
    pub game: GameType,

    /// Which version(s) of the game the plugin is compatible with.
    pub runtime_version: RuntimeVersion,

    /// The plugin's name.
    pub name: Option<String>,

    /// The plugin's authors.
    pub authors: Option<Vec<String>>,

    /// The plugin's description.
    pub description: Option<String>,

    /// The plugin's dependencies on other plugins.
    pub dependencies: Option<Vec<PluginDependency>>,
}

impl PluginMeta {
    pub fn builder(
        id: &str,
        version: Version
    ) -> PluginMetaBuilder {
        PluginMetaBuilder::new(id, version)
    }
}

pub struct PluginMetaBuilder {
    id: String,
    version: Version,
    name: Option<String>,
    authors: Option<Vec<String>>,
    description: Option<String>,
    game: GameType,
    runtime_version: RuntimeVersion,
    dependencies: Option<Vec<PluginDependency>>,
}

impl PluginMetaBuilder {
    fn new(
        id: &str,
        version: Version,
    ) -> Self {
        Self {
            id: id.to_string(),
            version,
            game: GameType::GameIndependent,
            runtime_version: RuntimeVersion::VersionIndependent,
            name: None,
            authors: None,
            description: None,
            dependencies: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn authors(mut self, authors: Vec<&str>) -> Self {
        self.authors = Some(authors.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn game(mut self, game: GameType) -> Self {
        self.game = game;
        self
    }

    pub fn runtime_version(mut self, runtime_version: RuntimeVersion) -> Self {
        self.runtime_version = runtime_version;
        self
    }

    pub fn dependencies(mut self, dependencies: Vec<PluginDependency>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    pub fn build(self) -> PluginMeta {
        PluginMeta {
            id: self.id,
            version: self.version,
            game: self.game,
            runtime_version: self.runtime_version,
            name: self.name,
            authors: self.authors,
            description: self.description,
            dependencies: self.dependencies,
        }
    }
}

pub trait Plugin {
    fn meta(&self) -> PluginMeta;

    /// Run as soon as the load order has been finalized.
    fn early_init(&self) {}
}

#[derive(Debug, Clone)]
pub struct CauldronEnv {}

impl CauldronEnv {
    #[doc(hidden)]
    pub fn new() -> Self {
        CauldronEnv {}
    }
}

unsafe impl Sync for CauldronEnv {}
unsafe impl Send for CauldronEnv {}

pub trait PluginOps: Plugin {
    fn env() -> &'static CauldronEnv;

    #[doc(hidden)]
    fn env_lock() -> &'static OnceLock<Box<CauldronEnv>>;
    #[doc(hidden)]
    fn init();
}

impl<P> PluginOps for P
where
    P: Plugin,
{
    fn env() -> &'static CauldronEnv {
        Self::env_lock().get().unwrap()
    }

    fn env_lock() -> &'static OnceLock<Box<CauldronEnv>> {
        static ENV: OnceLock<Box<CauldronEnv>> = OnceLock::new();
        &ENV
    }

    fn init() {
        // Self::env_lock().set(Box::new(env)).unwrap();
    }
}

pub enum PluginMainReason {
    Load,
    Unload,
}

#[macro_export]
macro_rules! define_plugin {
    ($t:ty, $f:expr) => {
        #[cfg(not(test))]
        mod __cauldron_api {
            use super::*;

            #[no_mangle]
            unsafe extern "C" fn __cauldron_api__plugin(
            ) -> Box<dyn $crate::Plugin + Send + Sync + 'static> {
                Box::<$t>::new($f)
            }

            #[no_mangle]
            unsafe extern "C" fn __cauldron_api__main(reason: $crate::PluginMainReason) -> () {
                match reason {
                    $crate::PluginMainReason::Load => {
                        <$t as $crate::PluginOps>::init();
                    }
                    _ => {}
                }
            }
        }
    };
}

pub mod log {
    pub use log::Level;

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

#[no_mangle]
unsafe extern "system" fn DllMain(_: isize, reason: u32, _: usize) -> bool {
    if reason == 1u32
    /*DLL_PROCESS_ATTACH*/
    {
        // todo: check if we need to spawn a thread for this
        on_dll_attach();
    }
    true
}
