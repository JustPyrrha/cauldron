#![feature(decl_macro)]

mod types;

use crate::types::{as_compound, as_container, as_enum, as_pointer, as_primitive, rtti_display_name, rtti_name, RTTIKind, RTTI};
use cauldron::prelude::*;
use cauldron::version::GameType;
use cauldron::{define_plugin, Plugin, PluginMeta};
use libc::uintptr_t;
use pelite::pattern::{parse, save_len};
use pelite::pe64::Pe;
use pelite::pe64::PeView;
use pelite::ImageMap;
use std::env::current_exe;
use std::fs::File;
use std::mem::transmute;
use std::{cmp, slice};
use std::ffi::{c_void, CStr};

pub struct PulsePlugin;

impl Plugin for PulsePlugin {
    fn meta(self) -> PluginMeta {
        PluginMeta {
            id: String::from("pulse"),
            version: Version::parse("0.1.0").unwrap(),
            game: GameType::HorizonForbiddenWest,
            optional: Default::default(),
        }
    }

    fn early_init(&self) {
        let mut file = File::create("rtti.txt").unwrap();
        let image = ImageMap::open(current_exe().unwrap().to_str().unwrap()).unwrap();
        let view = PeView::from_bytes(&image).unwrap();
        let data_header = view.section_headers().by_name(".data").unwrap();
        let rdata_header = view.section_headers().by_name(".rdata").unwrap();

        let is_data_segment = |ptr: u32| {
            ptr >= data_header.VirtualAddress
                && ptr < u32::wrapping_add(data_header.VirtualAddress, data_header.SizeOfRawData)
        };

        let is_rdata_segment = |ptr: u32| {
            ptr >= rdata_header.VirtualAddress
                && ptr < u32::wrapping_add(rdata_header.VirtualAddress, rdata_header.SizeOfRawData)
        };

        let mut types: Vec<*const RTTI> = Vec::new();

        // https://yara.readthedocs.io/en/v3.7.0/writingrules.html#hexadecimal-strings
        let pat = parse("FF FF FF FF [4]").unwrap();
        let mut results = view.scanner().matches(
            &pat,
            data_header.VirtualAddress
                ..u32::wrapping_add(data_header.VirtualAddress, data_header.SizeOfRawData),
        );
        // let mut results = view.scanner().matches(&pat, view.headers().image_range());
        let mut save = [0; 32];
        let save_len = cmp::min(save_len(&pat), save.len());
        unsafe {
            while results.next(&mut save[..save_len]) {
                { use std::io::Write; writeln!(file, "{:x}: {:?}", save[0], save).unwrap(); }
                let rtti = transmute::<*const uintptr_t, *const RTTI>(save[0] as *const _);
                { use std::io::Write; writeln!(file, "{}", (*rtti).kind).unwrap(); }

                if let Some(container) = as_container(rtti) {
                    if !is_data_segment((*container).container_type as u32)
                        || !is_data_segment((*container).item_type as u32)
                    {
                        continue;
                    }
                } else if let Some(_enum) = as_enum(rtti) {
                    if !is_data_segment((*_enum).name as u32)
                        || !is_data_segment((*_enum).values as u32)
                    {
                        continue;
                    }
                } else if let Some(class) = as_compound(rtti) {
                    if !is_rdata_segment((*class).name as u32) || (*class).alignment <= 0 {
                        continue;
                    }
                } else {
                    continue;
                }

                scan_recursively(rtti, &mut types);
            }

            types.iter().for_each(|rtti| {
                export_type(*rtti, &mut file);
            });
        }
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_plugin!(PulsePlugin, PulsePlugin {});

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
    } else if let Some(class) = as_compound(rtti) {
        for base in slice::from_raw_parts((*class).bases, (*class).num_bases as usize) {
            scan_recursively(base.base, types);
        }
        for attr in slice::from_raw_parts((*class).attributes, (*class).num_attributes as usize) {
            scan_recursively(attr.base, types);
        }
        for msg_handler in slice::from_raw_parts(
            (*class).message_handlers,
            (*class).num_message_handlers as usize,
        ) {
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

            for msg_handler in slice::from_raw_parts((*class).message_handlers, (*class).num_message_handlers as usize) {
                writeln!(file, "\t\t{} ({:p})", rtti_name(msg_handler.message), msg_handler.message).unwrap();
            }
        }

        if (*class).num_bases > 0 {
            writeln!(
                file,
                "\tbases: ({:p}/{})",
                (*class).bases,
                (*class).num_bases
            ).unwrap();
            for base in slice::from_raw_parts((*class).bases, (*class).num_bases as usize) {
                writeln!(file, "\t\t{} ({})", rtti_display_name(base.base), base.offset).unwrap();
            }
        }

        if (*class).num_attributes > 0 {
            writeln!(
                file,
                "\tattributes: ({:p}/{})",
                (*class).attributes,
                (*class).num_attributes
            ).unwrap();

            for attr in slice::from_raw_parts((*class).attributes, (*class).num_attributes as usize) {
                if attr.base.is_null() {
                    writeln!(file, "\t\tcategory: {}", CStr::from_ptr(attr.name).to_str().unwrap()).unwrap();
                } else {
                    writeln!(
                        file,
                        "\t\t{} ({}):",
                        CStr::from_ptr(attr.name).to_str().unwrap(),
                        rtti_display_name(attr.base),
                    ).unwrap();
                    writeln!(
                        file,
                        "\t\t\toffset: {}",
                        attr.offset
                    ).unwrap();
                    writeln!(
                        file,
                        "\t\t\tflags: {}",
                        attr.flags
                    ).unwrap();
                    if !attr.min_value.is_null() {
                        writeln!(
                            file,
                            "\t\t\tmin: {}",
                            CStr::from_ptr(attr.min_value).to_str().unwrap()
                        ).unwrap();
                    }
                    if !attr.max_value.is_null() {
                        writeln!(
                            file,
                            "\t\t\tmax: {}",
                            CStr::from_ptr(attr.max_value).to_str().unwrap()
                        ).unwrap();
                    }
                    if !attr.getter.is_null() || !attr.setter.is_null() {
                        writeln!(
                            file,
                            "\t\t\tproperty: true"
                        ).unwrap();
                    }
                }
            }
        }
    } else if let Some(enum_) = as_enum(rtti) {
        writeln!(
            file,
            "\tsize: {}",
            (*enum_).size
        ).unwrap();
        writeln!(
            file,
            "\tvalues: ({:p}/{})",
            (*enum_).values,
            (*enum_).num_values
        ).unwrap();
        for value in slice::from_raw_parts((*enum_).values, (*enum_).num_values as usize) {
            if value.aliases[0].is_null() {
                writeln!(
                    file,
                    "\t\t{}: {}",
                    CStr::from_ptr(value.name).to_str().unwrap(),
                    value.value
                ).unwrap();
            } else {
                writeln!(
                    file,
                    "\t\t{}: {} ({})",
                    CStr::from_ptr(value.name).to_str().unwrap(),
                    value.value,
                    value.aliases.iter().filter(|a| !a.is_null()).map(|p| CStr::from_ptr(*p).to_str().unwrap()).collect::<Vec<_>>().join(", ")
                ).unwrap();
            }
        }
    } else if let Some(primitive) = as_primitive(rtti) {
        writeln!(file, "\tbase: {}", rtti_display_name((*primitive).base_type)).unwrap();
    }

    writeln!(file).unwrap();
}
