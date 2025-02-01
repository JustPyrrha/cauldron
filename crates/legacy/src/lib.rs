use cauldron::{define_cauldron_plugin, CauldronLoader, CauldronPlugin};
use libdecima::log;

pub struct LegacyCauldron {}

impl CauldronPlugin for LegacyCauldron {
    fn new() -> Self {
        LegacyCauldron {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        log!("That which is lost or forbidden.");
    }
}

unsafe impl Send for LegacyCauldron {}
unsafe impl Sync for LegacyCauldron {}

define_cauldron_plugin!(LegacyCauldron, include_str!("../legacy.cauldron.toml"));
