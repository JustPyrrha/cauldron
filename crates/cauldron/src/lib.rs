#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]
pub mod prelude;
pub mod version;
mod loader;

use semver::{Version, VersionReq};
use crate::loader::on_dll_attach;
use crate::version::{GameType, RuntimeVersion};

#[derive(Debug, Clone)]
pub struct PluginDependency {
    /// The ID of the plugin to depend on.
    pub id: String,
    /// The version requirements of the dependency.
    pub versions: VersionReq
}

impl PluginDependency {
    /// Creates a new plugin dependency spec.
    pub fn new(id: impl Into<String>, versions: VersionReq) -> Self {
        Self { id: id.into(), versions }
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

    /// Extra meta that isn't required.
    pub optional: OptionalPluginMeta
}

#[derive(Debug, Clone)]
pub struct OptionalPluginMeta {
    /// Which version(s) of the game the plugin is compatible with.
    pub runtime_version: RuntimeVersion,

    /// The plugin's name.
    pub name: String,

    /// The plugin's authors.
    pub authors: Vec<String>,

    /// The plugin's description.
    pub description: String,

    /// The plugin's dependencies on other plugins.
    pub dependencies: Vec<PluginDependency>
}

impl Default for OptionalPluginMeta {
    fn default() -> Self {
        Self {
            runtime_version: RuntimeVersion::VersionIndependent,
            name: String::new(),
            authors: Vec::new(),
            description: String::new(),
            dependencies: Vec::new(),
        }
    }
}

pub trait Plugin {
    fn meta() -> PluginMeta where Self: Sized;

    /// Run as soon as the load order has been finalized.
    fn early_init(&self) {}
}

#[macro_export]
macro_rules! define_plugin {
    ($t:ty, $f:expr) => {
        #[cfg(not(test))]
        mod __cauldron_api {
            use super::*;

            #[no_mangle]
            unsafe extern "C" fn __cauldron_api__plugin() -> Box<dyn $crate::Plugin + Send + Sync + 'static> {
                Box::<$t>::new($f)
            }
        }
    };
}

#[no_mangle]
unsafe extern "system" fn DllMain(_: isize, reason: u32, _: usize) -> bool {
    if reason == 1u32 /*DLL_PROCESS_ATTACH*/ {
        // todo: check if we need to spawn a thread for this
        on_dll_attach();
    }
    true
}


