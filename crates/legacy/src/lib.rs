mod hooks;
mod types;

use crate::hooks::init_hooks;
use cauldron::{CauldronLoader, CauldronPlugin, define_cauldron_plugin};
use libdecima::log;
use libdecima::mem::offsets::Offsets;

pub struct LegacyCauldron {}

impl CauldronPlugin for LegacyCauldron {
    fn new() -> Self {
        LegacyCauldron {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        log!("That which is lost or forbidden.");
        Offsets::setup();
        init_hooks();
        log!("nvm found it :3");
    }
}

unsafe impl Send for LegacyCauldron {}
unsafe impl Sync for LegacyCauldron {}

define_cauldron_plugin!(LegacyCauldron, include_str!("../legacy.cauldron.toml"));
