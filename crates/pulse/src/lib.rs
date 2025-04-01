#![feature(let_chains)]
#![allow(static_mut_refs)]

use crate::ida_export::ida_export;
use cauldron::{CauldronLoader, CauldronPlugin, define_cauldron_plugin};
use libdecima::log;
use libdecima::types::decima::core::factory_manager::FactoryManager;

mod ida_export;

pub struct PulsePlugin {}

impl CauldronPlugin for PulsePlugin {
    fn new() -> PulsePlugin {
        PulsePlugin {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        let Some(factory) = FactoryManager::get_instance() else {
            log!("error: failed to get FactoryManager instance");
            return;
        };
        log!("pulse", "found {} types.", factory.types.count);
        let types = factory.types.slice();
        let mut new_types = vec![];
        for ty in types {
            if !ty.value.is_null() {
                let ty = unsafe { &*ty.value };
                new_types.push(ty);
            }
        }

        ida_export(new_types).unwrap()
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_cauldron_plugin!(PulsePlugin, include_str!("../pulse.cauldron.toml"));
