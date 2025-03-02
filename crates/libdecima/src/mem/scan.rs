use crate::mem::{find_pattern, get_data_section, get_rdata_section};
use crate::types::decima::core::rtti::*;
use std::ffi::c_void;

pub unsafe fn scan_memory_for_types(rtti_scan_callback: fn(rtti: *const RTTI)) -> Vec<*const RTTI> {
    unsafe {
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
            if let Some(primitive) = rtti.as_atom() {
                if primitive.size == 0
                    || primitive.alignment == 0
                    || (!primitive.fn_constructor.is_null()
                        && !is_valid_ptr(primitive.fn_constructor as usize))
                    || (!primitive.fn_destructor.is_null()
                        && !is_valid_ptr(primitive.fn_destructor as usize))
                    || !is_valid_ptr(primitive.base_type as usize)
                    || !is_valid_ptr(primitive.type_name as usize)
                {
                    continue;
                }
            } else if let Some(enum_) = rtti.as_enum() {
                if enum_.size == 0
                    || !is_valid_ptr(enum_.type_name as usize)
                    || !is_valid_ptr(enum_.values as usize)
                {
                    continue;
                }
            } else if let Some(container) = rtti.as_container() {
                let container = &*container;
                if !is_valid_ptr(container.item_type as usize)
                    || !is_valid_ptr(container.container_type as usize)
                    || !is_valid_ptr((&*container.container_type).type_name as usize)
                // || (!(&*container.container_type).fn_constructor.is_null()
                //     && !is_valid_ptr((&*container.container_type).fn_constructor as usize))
                // || (!(&*container.container_type).fn_destructor.is_null()
                //     && !is_valid_ptr((&*container.container_type).fn_destructor as usize))
                {
                    continue;
                }
            } else if let Some(compound) = rtti.as_compound() {
                if !is_valid_ptr(compound.type_name as usize)
                    || (compound.bases_len > 0 && !is_valid_ptr(compound.bases as usize))
                    || (compound.attributes_len > 0
                        && !is_valid_ptr(compound.attributes_len as usize))
                    || (compound.message_handlers_len > 0
                        && !is_valid_ptr(compound.message_handlers as usize))
                    || (compound.message_order_entries_len > 0
                        && !is_valid_ptr(compound.message_order_entries as usize))
                    || (compound.ordered_attributes_len > 0
                        && !is_valid_ptr(compound.ordered_attributes as usize))
                {
                    continue;
                }
            } else {
                continue;
            }

            scan_recursively(rtti, &mut types, rtti_scan_callback);
        }

        types
    }
}

pub unsafe fn scan_recursively(
    rtti: *const RTTI,
    types: &mut Vec<*const RTTI>,
    callback: fn(rtti: *const RTTI),
) {
    unsafe {
        if rtti.is_null() || types.contains(&rtti) {
            return;
        }

        callback(rtti);
        types.push(rtti);

        let rtti = &*rtti;

        if let Some(container) = rtti.as_container() {
            scan_recursively(container.item_type, types, callback);
        } else if let Some(primitive) = rtti.as_atom() {
            scan_recursively(primitive.base_type as *const RTTI, types, callback);
        } else if let Some(compound) = rtti.as_compound() {
            for base in compound.bases() {
                scan_recursively(base.r#type, types, callback);
            }
            for attr in compound.attributes() {
                scan_recursively(attr.r#type, types, callback);
            }
            for msg_handler in compound.message_handlers() {
                scan_recursively(msg_handler.message, types, callback);
            }
        }
    }
}
