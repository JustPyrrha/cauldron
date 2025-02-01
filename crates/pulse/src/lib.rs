#![allow(static_mut_refs)]
#![feature(c_variadic)]
#![feature(core_intrinsics)]

mod json;

use cauldron::{define_cauldron_plugin, CauldronLoader, CauldronPlugin};
use libdecima::log;
use libdecima::mem::offsets::Offsets;
use libdecima::mem::{get_data_section, get_rdata_section, offset_from_instruction};
use libdecima::types::nixxes::log::NxLogImpl;
use libdecima::types::rtti::{as_atom, as_compound, as_container, as_pointer, RTTI};
use minhook::MhHook;
use once_cell::sync::OnceCell;
use std::ffi::{c_char, c_void, VaList};
use std::fs::{File, OpenOptions};
use std::slice;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F5, VK_F6};

static RTTI_FACTORY_REGISTER_TYPE: OnceCell<
    unsafe fn(factory: *mut c_void, rtti: *const RTTI) -> bool,
> = OnceCell::new();

static NX_LOG_IMPL_FN_LOG: OnceCell<FnNxLogImplLog> = OnceCell::new();

static mut FOUND_TYPES: OnceCell<Vec<*const RTTI>> = OnceCell::new();

pub struct PulsePlugin {}

impl CauldronPlugin for PulsePlugin {
    fn new() -> PulsePlugin {
        PulsePlugin {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
        // todo: rewrite when im more awake

        // Offsets::instance().setup();
        //
        // let register_type = unsafe {
        //     MhHook::new(
        //         *Offsets::instance()
        //             .resolve("RTTIFactory::RegisterType")
        //             .unwrap() as *mut _,
        //         rtti_factory_register_type_impl as *mut _,
        //     )
        //     .unwrap()
        // };
        // unsafe {
        //     RTTI_FACTORY_REGISTER_TYPE
        //         .set(std::mem::transmute(register_type.trampoline()))
        //         .unwrap();
        // }
        //
        // log!("scanning for rtti structures...");
        //
        // unsafe {
        //     FOUND_TYPES.get_or_init(|| Vec::new());
        //     FOUND_TYPES
        //         .get_mut()
        //         .unwrap()
        //         .append(&mut scan_memory_for_types());
        // }
        //
        // log!(format!("scan finished, found {} in-memory types.", unsafe {
        //     FOUND_TYPES.get().unwrap().len()
        // })
        // .as_str());
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_cauldron_plugin!(PulsePlugin, include_str!("../pulse.cauldron.toml"));

type FnNxLogImplLog = unsafe extern "C" fn(
    instance: *mut NxLogImpl,
    category: *const c_char,
    format: *const c_char,
    format_args: VaList,
);

static LOGGED: OnceCell<bool> = OnceCell::new();
unsafe fn nx__nx_log_impl__fn_log_impl(
    instance: *mut NxLogImpl,
    category: *const c_char,
    format: *const c_char,
    format_args: VaList,
) {
    std::intrinsics::breakpoint();

    LOGGED.get_or_init(|| {
        // shitty call once lol
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("instance.log")
            .unwrap();
        use std::io::Write as _;
        writeln!(file, "{:p}", instance).unwrap();

        true
    });

    (NX_LOG_IMPL_FN_LOG.get().unwrap())(instance, category, format, format_args)
}

unsafe fn rtti_factory_register_type_impl(factory: *mut c_void, rtti: *const RTTI) -> bool {
    let result = (RTTI_FACTORY_REGISTER_TYPE.get().unwrap())(factory, rtti);

    if result {
        scan_recursively(rtti, unsafe { FOUND_TYPES.get_mut().unwrap() });
    }

    result
}

unsafe fn scan_memory_for_types() -> Vec<*const RTTI> {
    vec![]

    // todo: rewrite pattern16 pattern

    // let (data_start, data_end) = get_data_section().unwrap_or((0, 0));
    // let (rdata_start, rdata_end) = get_rdata_section().unwrap_or((0, 0));
    //
    // let is_valid_ptr = |ptr: usize| {
    //     if ptr == 0 {
    //         false
    //     } else {
    //         (ptr >= data_start && ptr < data_end) || (ptr >= rdata_start && ptr < rdata_end)
    //     }
    // };
    //
    // let mut types: Vec<*const RTTI> = Vec::new();
    //
    // let mut current: *const c_void = data_start as *const c_void;
    // loop {
    //     let rtti_ptr = find_pattern(current as *const u8, data_end, "FF FF FF FF [00000???]");
    //     let Some(rtti_ptr) = rtti_ptr else {
    //         break;
    //     };
    //
    //     current = unsafe { rtti_ptr.add(5) };
    //     let rtti = unsafe { &*(rtti_ptr as *const RTTI) };
    //     if let Some(primitive) = as_atom(rtti) {
    //         let primitive = &*primitive;
    //         if primitive.size == 0
    //             || primitive.alignment == 0
    //             || (!primitive.fn_constructor.is_null()
    //                 && !is_valid_ptr(primitive.fn_constructor as usize))
    //             || (!primitive.fn_destructor.is_null()
    //                 && !is_valid_ptr(primitive.fn_destructor as usize))
    //             || !is_valid_ptr(primitive.parent_type as usize)
    //             || !is_valid_ptr(primitive.type_name as usize)
    //         {
    //             continue;
    //         }
    //     } else if let Some(enum_) = as_enum(rtti) {
    //         let enum_ = &*enum_;
    //         if enum_.size == 0
    //             || !is_valid_ptr(enum_.type_name as usize)
    //             || !is_valid_ptr(enum_.values as usize)
    //         {
    //             continue;
    //         }
    //     } else if let Some(container) = as_container(rtti) {
    //         let container = &*container;
    //         if !is_valid_ptr(container.item_type as usize)
    //             || !is_valid_ptr(container.container_type as usize)
    //             || !is_valid_ptr((&*container.container_type).type_name as usize)
    //             || (!(&*container.container_type).fn_constructor.is_null()
    //                 && !is_valid_ptr((&*container.container_type).fn_constructor as usize))
    //             || (!(&*container.container_type).fn_destructor.is_null()
    //                 && !is_valid_ptr((&*container.container_type).fn_destructor as usize))
    //         {
    //             continue;
    //         }
    //     } else if let Some(pointer) = as_pointer(rtti) {
    //         let pointer = &*pointer;
    //         if !is_valid_ptr(pointer.item_type as usize)
    //             || !is_valid_ptr(pointer.pointer_type as usize)
    //             || !is_valid_ptr((&*pointer.pointer_type).type_name as usize)
    //             || !is_valid_ptr(pointer.type_name as usize)
    //             || (!(&*pointer.pointer_type).fn_constructor.is_null()
    //                 && !is_valid_ptr((&*pointer.pointer_type).fn_constructor as usize))
    //             || (!(&*pointer.pointer_type).fn_destructor.is_null()
    //                 && !is_valid_ptr((&*pointer.pointer_type).fn_destructor as usize))
    //         {
    //             continue;
    //         }
    //     } else if let Some(compound) = as_compound(rtti) {
    //         let compound = &*compound;
    //         if !is_valid_ptr(compound.type_name as usize)
    //             || (compound.num_bases > 0 && !is_valid_ptr(compound.bases as usize))
    //             || (compound.num_attrs > 0 && !is_valid_ptr(compound.attrs as usize))
    //             || (compound.num_message_handlers > 0
    //                 && !is_valid_ptr(compound.message_handlers as usize))
    //             || (compound.num_ordered_attrs > 0
    //                 && !is_valid_ptr(compound.ordered_attrs as usize))
    //         {
    //             continue;
    //         }
    //     } else {
    //         continue;
    //     }
    //
    //     scan_recursively(rtti, &mut types);
    // }
    //
    // types
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
