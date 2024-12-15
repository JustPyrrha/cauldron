#![feature(decl_macro)]

mod memory;
mod types;
mod ida;

use crate::memory::{get_data_section, get_rdata_section};
use crate::types::{as_compound, as_container, as_enum, as_pointer, as_primitive, rtti_display_name, rtti_name, RTTIContainerData, RTTIKind, RTTIPointerData, RTTI};
use cauldron::prelude::*;
use cauldron::version::GameType;
use cauldron::{define_plugin, info, Plugin, PluginMeta};
use pattern16::Pat16_scan;
use std::ffi::{c_void, CStr};
use std::fs::File;
use std::slice;
use crate::ida::export_ida_type;

pub struct PulsePlugin {}

impl Plugin for PulsePlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::builder("pulse", Version::parse(env!("CARGO_PKG_VERSION")).unwrap())
            .game(GameType::HorizonForbiddenWest)
            .build()
    }

    fn early_init(&self) {
        info!("pulse: scanning for rtti structures...");
        let mut file = File::create("rtti.txt").unwrap();
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
            let rtti_ptr = unsafe { Pat16_scan(current, data_end) };
            if rtti_ptr.is_null() {
                break;
            }
            current = unsafe { rtti_ptr.add(5) };
            let rtti = unsafe { &*(rtti_ptr as *const RTTI) };
            unsafe {
                if let Some(primitive) = as_primitive(rtti) {
                    let primitive = &*primitive;
                    if primitive.size == 0
                        || primitive.alignment == 0
                        || (!primitive.constructor.is_null()
                            && !is_valid_ptr(primitive.constructor as usize))
                        || (!primitive.destructor.is_null()
                            && !is_valid_ptr(primitive.destructor as usize))
                        || !is_valid_ptr(primitive.base_type as usize)
                        || !is_valid_ptr(primitive.name as usize)
                    {
                        continue;
                    }
                } else if let Some(enum_) = as_enum(rtti) {
                    let enum_ = &*enum_;
                    if enum_.size == 0
                        || !is_valid_ptr(enum_.name as usize)
                        || !is_valid_ptr(enum_.values as usize)
                    {
                        continue;
                    }
                } else if let Some(container) = as_container(rtti) {
                    let container = &*container;
                    if !is_valid_ptr(container.item_type as usize)
                        || !is_valid_ptr(container.container_type as usize)
                        || !is_valid_ptr((&*container.container_type).name as usize)
                        || (!(&*container.container_type).constructor.is_null()
                            && !is_valid_ptr((&*container.container_type).constructor as usize))
                        || (!(&*container.container_type).destructor.is_null()
                            && !is_valid_ptr((&*container.container_type).destructor as usize))
                    {
                        continue;
                    }
                } else if let Some(pointer) = as_pointer(rtti) {
                    let pointer = &*pointer;
                    if !is_valid_ptr(pointer.item_type as usize)
                        || !is_valid_ptr(pointer.pointer_type as usize)
                        || !is_valid_ptr((&*pointer.pointer_type).name as usize)
                        || (!(&*pointer.pointer_type).constructor.is_null()
                            && !is_valid_ptr((&*pointer.pointer_type).constructor as usize))
                        || (!(&*pointer.pointer_type).destructor.is_null()
                            && !is_valid_ptr((&*pointer.pointer_type).destructor as usize))
                    {
                        continue;
                    }
                } else if let Some(compound) = as_compound(rtti) {
                    let compound = &*compound;
                    if !is_valid_ptr(compound.name as usize)
                        || (compound.num_bases > 0 && !is_valid_ptr(compound.bases as usize))
                        || (compound.num_attributes > 0
                            && !is_valid_ptr(compound.attributes as usize))
                        || (compound.num_message_handlers > 0
                            && !is_valid_ptr(compound.message_handlers as usize))
                    {
                        continue;
                    }
                } else {
                    continue;
                }

                scan_recursively(rtti, &mut types);
            }
        }

        info!("pulse: scan finished, found {}. exporting...", types.len());
        types.iter().for_each(|rtti| unsafe {
            export_type(*rtti, &mut file);
        });
        {
            use std::io::Write;
            info!("pulse: exporting types for ida...");
            let mut ida_file = File::create("hfw_ggrtti.idc").unwrap();
            let mut existing_containers = Vec::<*mut RTTIContainerData>::new();
            let mut existing_pointers = Vec::<*mut RTTIPointerData>::new();
            writeln!(ida_file, "#include <idc.idc>\n\nstatic main() {{").unwrap();
            types.iter().for_each(|rtti| unsafe {
                export_ida_type(*rtti, &mut ida_file, &mut existing_containers, &mut existing_pointers).unwrap();
            });
            writeln!(ida_file, "}}").unwrap();
        }
        info!("pulse: done");
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_plugin!(PulsePlugin, PulsePlugin { });

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
    } else if let Some(primitive) = as_primitive(rtti) {
        scan_recursively((*primitive).base_type, types);
    } else if let Some(compound) = as_compound(rtti) {
        let compound = &*compound;
        for base in compound.bases() {
            scan_recursively(base.base, types);
        }
        for attr in compound.attributes() {
            scan_recursively(attr.base, types);
        }
        for msg_handler in compound.message_handlers() {
            scan_recursively(msg_handler.message, types);
        }
    }
}

unsafe fn export_type<W: std::io::Write>(rtti: *const RTTI, file: &mut W) {
    if (*rtti).kind == RTTIKind::Pointer
        || (*rtti).kind == RTTIKind::Container
        || (*rtti).kind == RTTIKind::POD
    {
        return;
    }

    writeln!(file, "{} ({})", rtti_display_name(rtti), (*rtti).kind).unwrap();

    if let Some(class) = as_compound(rtti) {
        writeln!(file, "\tversion: {}", (*class).version).unwrap();
        writeln!(file, "\tflags: {}", (*class).flags).unwrap();

        if (*class).num_message_handlers > 0 {
            writeln!(
                file,
                "\tmessage_handlers: ({:p}/{})",
                (*class).message_handlers,
                (*class).num_message_handlers
            )
            .unwrap();

            for msg_handler in slice::from_raw_parts(
                (*class).message_handlers,
                (*class).num_message_handlers as usize,
            ) {
                writeln!(
                    file,
                    "\t\t{} ({:p})",
                    rtti_name(msg_handler.message),
                    msg_handler.message
                )
                .unwrap();
            }
        }

        if (*class).num_bases > 0 {
            writeln!(
                file,
                "\tbases: ({:p}/{})",
                (*class).bases,
                (*class).num_bases
            )
            .unwrap();
            for base in slice::from_raw_parts((*class).bases, (*class).num_bases as usize) {
                writeln!(
                    file,
                    "\t\t{} ({})",
                    rtti_display_name(base.base),
                    base.offset
                )
                .unwrap();
            }
        }

        if (*class).num_attributes > 0 {
            writeln!(
                file,
                "\tattributes: ({:p}/{})",
                (*class).attributes,
                (*class).num_attributes
            )
            .unwrap();

            for attr in slice::from_raw_parts((*class).attributes, (*class).num_attributes as usize)
            {
                if attr.base.is_null() {
                    writeln!(
                        file,
                        "\t\tcategory: {}",
                        CStr::from_ptr(attr.name).to_str().unwrap()
                    )
                    .unwrap();
                } else {
                    writeln!(
                        file,
                        "\t\t{} ({}):",
                        CStr::from_ptr(attr.name).to_str().unwrap(),
                        rtti_display_name(attr.base),
                    )
                    .unwrap();
                    writeln!(file, "\t\t\toffset: {}", attr.offset).unwrap();
                    writeln!(file, "\t\t\tflags: {}", attr.flags).unwrap();
                    if !attr.min_value.is_null() {
                        writeln!(
                            file,
                            "\t\t\tmin: {}",
                            CStr::from_ptr(attr.min_value).to_str().unwrap()
                        )
                        .unwrap();
                    }
                    if !attr.max_value.is_null() {
                        writeln!(
                            file,
                            "\t\t\tmax: {}",
                            CStr::from_ptr(attr.max_value).to_str().unwrap()
                        )
                        .unwrap();
                    }
                    if !attr.getter.is_null() || !attr.setter.is_null() {
                        writeln!(file, "\t\t\tproperty: true").unwrap();
                    }
                }
            }
        }
    } else if let Some(enum_) = as_enum(rtti) {
        writeln!(file, "\tsize: {}", (*enum_).size).unwrap();
        writeln!(
            file,
            "\tvalues: ({:p}/{})",
            (*enum_).values,
            (*enum_).num_values
        )
        .unwrap();
        for value in slice::from_raw_parts((*enum_).values, (*enum_).num_values as usize) {
            if value.aliases[0].is_null() {
                writeln!(
                    file,
                    "\t\t{}: {}",
                    CStr::from_ptr(value.name).to_str().unwrap(),
                    value.value
                )
                .unwrap();
            } else {
                writeln!(
                    file,
                    "\t\t{}: {} ({})",
                    CStr::from_ptr(value.name).to_str().unwrap(),
                    value.value,
                    value
                        .aliases
                        .iter()
                        .filter(|a| !a.is_null())
                        .map(|p| CStr::from_ptr(*p).to_str().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .unwrap();
            }
        }
    } else if let Some(primitive) = as_primitive(rtti) {
        writeln!(
            file,
            "\tbase: {}",
            rtti_display_name((*primitive).base_type)
        )
        .unwrap();
    }

    writeln!(file).unwrap();
}