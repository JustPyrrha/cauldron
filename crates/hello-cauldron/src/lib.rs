use cauldron::{define_cauldron_plugin, info, CauldronLoader, CauldronPlugin};

pub struct HelloCauldron {}

impl CauldronPlugin for HelloCauldron {
    fn new() -> Self {
        HelloCauldron {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        info!("Hello Cauldron!");
    }

    fn on_deinit(&self) {
        info!("Goodbye Cauldron!");
    }
}

unsafe impl Send for HelloCauldron {}
unsafe impl Sync for HelloCauldron {}

define_cauldron_plugin!(HelloCauldron, include_str!("../hello.cauldron.toml"));
