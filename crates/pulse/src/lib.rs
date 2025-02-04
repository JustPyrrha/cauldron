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
        scan_recursively(rtti, unsafe { FOUND_TYPES.get_mut().unwrap() });
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
            .append(&mut scan_memory_for_types());
    }

    log!("pulse", "scan finished, found {} types.", unsafe {
        FOUND_TYPES.get().unwrap().len()
    });

    log!("exporting types...");
    export_types_json(unsafe { FOUND_TYPES.get().unwrap() });
    log!("done.")
}

unsafe fn scan_memory_for_types() -> Vec<*const RTTI> {
    let (data_start, data_end) = get_data_section().unwrap_or((0, 0));
    let (rdata_start, rdata_end) = get_rdata_section().unwrap_or((0, 0));

    let is_valid_ptr = |ptr: usize| {
        if ptr == 0 {
            false
        } else {
            (ptr >= data_start && ptr < data_end) || (ptr >= rdata_start && ptr < rdata_end)
        }
    };

    let mut types: Vec<*const RTTI> = Vec::new();

    let mut current: *const c_void = data_start as *const c_void;
    loop {
        let rtti_ptr = find_pattern(current as *mut u8, data_end, "FF FF FF FF ? ? ? ?");
        let Ok(rtti_ptr) = rtti_ptr else {
            break;
        };
        let rtti_ptr = rtti_ptr as *const c_void;

        current = unsafe { rtti_ptr.add(5) };
        let rtti = unsafe { &*(rtti_ptr as *const RTTI) };
        if let Some(primitive) = as_atom(rtti) {
            let primitive = &*primitive;
            if primitive.size == 0
                || primitive.alignment == 0
                || (!primitive.fn_constructor.is_null()
                    && !is_valid_ptr(primitive.fn_constructor as usize))
                || (!primitive.fn_destructor.is_null()
                    && !is_valid_ptr(primitive.fn_destructor as usize))
                || !is_valid_ptr(primitive.parent_type as usize)
                || !is_valid_ptr(primitive.type_name as usize)
            {
                continue;
            }
        } else if let Some(enum_) = as_enum(rtti) {
            let enum_ = &*enum_;
            if enum_.size == 0
                || !is_valid_ptr(enum_.type_name as usize)
                || !is_valid_ptr(enum_.values as usize)
            {
                continue;
            }
        } else if let Some(container) = as_container(rtti) {
            let container = &*container;
            if !is_valid_ptr(container.item_type as usize)
                || !is_valid_ptr(container.container_type as usize)
                || !is_valid_ptr((&*container.container_type).type_name as usize)
                || (!(&*container.container_type).fn_constructor.is_null()
                    && !is_valid_ptr((&*container.container_type).fn_constructor as usize))
                || (!(&*container.container_type).fn_destructor.is_null()
                    && !is_valid_ptr((&*container.container_type).fn_destructor as usize))
            {
                continue;
            }
        } else if let Some(pointer) = as_pointer(rtti) {
            let pointer = &*pointer;
            if !is_valid_ptr(pointer.item_type as usize)
                || !is_valid_ptr(pointer.pointer_type as usize)
                || !is_valid_ptr((&*pointer.pointer_type).type_name as usize)
                || !is_valid_ptr(pointer.type_name as usize)
                || (!(&*pointer.pointer_type).fn_constructor.is_null()
                    && !is_valid_ptr((&*pointer.pointer_type).fn_constructor as usize))
                || (!(&*pointer.pointer_type).fn_destructor.is_null()
                    && !is_valid_ptr((&*pointer.pointer_type).fn_destructor as usize))
            {
                continue;
            }
        } else if let Some(compound) = as_compound(rtti) {
            let compound = &*compound;
            if !is_valid_ptr(compound.type_name as usize)
                || (compound.num_bases > 0 && !is_valid_ptr(compound.bases as usize))
                || (compound.num_attrs > 0 && !is_valid_ptr(compound.attrs as usize))
                || (compound.num_message_handlers > 0
                    && !is_valid_ptr(compound.message_handlers as usize))
                || (compound.num_ordered_attrs > 0
                    && !is_valid_ptr(compound.ordered_attrs as usize))
            {
                continue;
            }
        } else {
            continue;
        }

        scan_recursively(rtti, &mut types);
    }

    types
}

unsafe fn scan_recursively(rtti: *const RTTI, types: &mut Vec<*const RTTI>) {
    if rtti.is_null() || types.contains(&rtti) {
        return;
    }

    types.push(rtti);

    if let Some(container) = as_container(rtti) {
        scan_recursively((*container).item_type, types);
    }
    if let Some(pointer) = as_pointer(rtti) {
        scan_recursively((*pointer).item_type, types);
    } else if let Some(primitive) = as_atom(rtti) {
        scan_recursively(std::mem::transmute((*primitive).parent_type), types);
    } else if let Some(compound) = as_compound(rtti) {
        let compound = &*compound;
        for base in compound.bases() {
            scan_recursively(std::mem::transmute(base.r#type), types);
        }
        for attr in compound.attributes() {
            scan_recursively(attr.r#type, types);
        }
        for msg_handler in compound.message_handlers() {
            scan_recursively(msg_handler.message, types);
        }
    }
}
