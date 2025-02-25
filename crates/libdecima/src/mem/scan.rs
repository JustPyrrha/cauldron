use crate::mem::{find_pattern, get_data_section, get_rdata_section};
use crate::types::rtti::{as_atom, as_compound, as_container, as_enum, as_pointer, RTTI};
use std::ffi::c_void;

pub unsafe fn scan_memory_for_types(rtti_scan_callback: fn(rtti: *const RTTI)) -> Vec<*const RTTI> { unsafe {
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

        current = rtti_ptr.add(5);
        let rtti = &*(rtti_ptr as *const RTTI);
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

        scan_recursively(rtti, &mut types, rtti_scan_callback);
    }

    types
}}

pub unsafe fn scan_recursively(
    rtti: *const RTTI,
    types: &mut Vec<*const RTTI>,
    callback: fn(rtti: *const RTTI),
) { unsafe {
    if rtti.is_null() || types.contains(&rtti) {
        return;
    }

    callback(rtti);
    types.push(rtti);

    if let Some(container) = as_container(rtti) {
        scan_recursively((*container).item_type, types, callback);
    }
    if let Some(pointer) = as_pointer(rtti) {
        scan_recursively((*pointer).item_type, types, callback);
    } else if let Some(primitive) = as_atom(rtti) {
        scan_recursively(
            std::mem::transmute((*primitive).parent_type),
            types,
            callback,
        );
    } else if let Some(compound) = as_compound(rtti) {
        let compound = &*compound;
        for base in compound.bases() {
            scan_recursively(std::mem::transmute(base.r#type), types, callback);
        }
        for attr in compound.attributes() {
            scan_recursively(attr.r#type, types, callback);
        }
        for msg_handler in compound.message_handlers() {
            scan_recursively(msg_handler.message, types, callback);
        }
    }
}}
