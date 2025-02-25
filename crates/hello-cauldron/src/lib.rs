use cauldron::{CauldronLoader, CauldronPlugin, define_cauldron_plugin};
use libdecima::log;

pub struct HelloCauldron {}

impl CauldronPlugin for HelloCauldron {
    fn new() -> Self {
        HelloCauldron {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        log!("Hello Cauldron!");
    }

    fn on_deinit(&self) {
        log!("Goodbye Cauldron!");
    }
}

unsafe impl Send for HelloCauldron {}
unsafe impl Sync for HelloCauldron {}

define_cauldron_plugin!(HelloCauldron, include_str!("../hello.cauldron.toml"));
