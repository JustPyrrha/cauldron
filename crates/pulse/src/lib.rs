use log::info;
use cauldron::{define_plugin, Plugin, PluginMeta};
use cauldron::prelude::*;
use cauldron::version::GameType;

pub struct PulsePlugin;

impl Plugin for PulsePlugin {
    fn meta() -> PluginMeta {
        PluginMeta {
            id: String::from("pulse"),
            version: Version::parse("0.1.0").unwrap(),
            game: GameType::GameIndependent,
            optional: Default::default(),
        }
    }

    fn early_init(&self) {
        info!("pulse: hello early init!");
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}


define_plugin!(PulsePlugin, PulsePlugin { });