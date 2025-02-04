#![allow(static_mut_refs)]

mod json;

use crate::json::export_types_json;
use cauldron::{define_cauldron_plugin, CauldronLoader, CauldronPlugin};
use libdecima::log;
use libdecima::mem::offsets::Offsets;
use libdecima::mem::{find_pattern, get_data_section, get_rdata_section};
use libdecima::types::rtti::{as_atom, as_compound, as_container, as_enum, as_pointer, RTTI};
use minhook::MhHook;
use once_cell::sync::OnceCell;
use std::ffi::c_void;
use libdecima::mem::scan::{scan_memory_for_types, scan_recursively};

static RTTI_FACTORY_REGISTER_TYPE: OnceCell<
    unsafe fn(factory: *mut c_void, rtti: *const RTTI) -> bool,
> = OnceCell::new();
static RTTI_FACTORY_REGISTER_ALL_TYPES: OnceCell<unsafe fn()> = OnceCell::new();

static mut FOUND_TYPES: OnceCell<Vec<*const RTTI>> = OnceCell::new();

pub struct PulsePlugin {}

impl CauldronPlugin for PulsePlugin {
    fn new() -> PulsePlugin {
        PulsePlugin {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        Offsets::setup();

        let register_type = unsafe {
            MhHook::new(
                *Offsets::resolve_raw("RTTIFactory::RegisterType").unwrap() as *mut _,
                rtti_factory_register_type_impl as *mut _,
            )
            .unwrap()
        };

        let register_all_types = unsafe {
            MhHook::new(
                *Offsets::resolve_raw("RTTIFactory::RegisterAllTypes").unwrap() as *mut _,
                rtti_factory_register_all_types_impl as *mut _,
            )
            .unwrap()
        };
        unsafe {
            RTTI_FACTORY_REGISTER_TYPE
                .set(std::mem::transmute(register_type.trampoline()))
                .unwrap();
            RTTI_FACTORY_REGISTER_ALL_TYPES
                .set(std::mem::transmute(register_all_types.trampoline()))
                .unwrap()
        }
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_cauldron_plugin!(PulsePlugin, include_str!("../pulse.cauldron.toml"));

unsafe fn rtti_factory_register_type_impl(factory: *mut c_void, rtti: *const RTTI) -> bool {
    FOUND_TYPES.get_or_init(|| Vec::new());
    let result = (RTTI_FACTORY_REGISTER_TYPE.get().unwrap())(factory, rtti);

    if result {
        scan_recursively(rtti, unsafe { FOUND_TYPES.get_mut().unwrap() }, |_|{});
    }

    result
}

unsafe fn rtti_factory_register_all_types_impl() {
    (RTTI_FACTORY_REGISTER_ALL_TYPES.get().unwrap())();

    log!("scanning for rtti structures...");

    unsafe {
        FOUND_TYPES.get_or_init(|| Vec::new());
        FOUND_TYPES
            .get_mut()
            .unwrap()
            .append(&mut scan_memory_for_types(|_|{}));
    }

    log!("pulse", "scan finished, found {} types.", unsafe {
        FOUND_TYPES.get().unwrap().len()
    });

    log!("exporting types...");
    export_types_json(unsafe { FOUND_TYPES.get().unwrap() });
    log!("done.")
}