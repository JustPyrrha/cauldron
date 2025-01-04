#![feature(decl_macro)]
#![feature(extern_types)]

mod ida;
mod memory;
mod types;

pub mod p_core;

use crate::ida::export_ida_type;
use crate::memory::{
    find_offset_from, find_pattern, get_data_section, get_module, get_rdata_section,
};
use crate::p_core::array::GGArray;
use crate::types::{
    as_compound, as_container, as_enum, as_pointer, as_primitive, rtti_display_name, rtti_name,
    ExportedSymbolsGroup, RTTIContainerData, RTTIKind, RTTIPointerData, RTTI,
};
use cauldron::version::CauldronGameType;
use cauldron::{debug, define_cauldron_plugin, info, CauldronLoader, CauldronPlugin};
use std::ffi::{c_void, CStr};
use std::fs::File;
use std::slice;
use std::sync::OnceLock;

pub struct PulsePlugin {}

impl CauldronPlugin for PulsePlugin {
    fn new() -> PulsePlugin {
        PulsePlugin {}
    }

    fn on_init(&self, _loader: &CauldronLoader) {
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
            let rtti_ptr = find_pattern(current as *const u8, data_end, "FF FF FF FF [00000???]");
            let Some(rtti_ptr) = rtti_ptr else {
                break;
            };

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
        // info!("pulse: exporting symbols");
        // export_symbols();
        // info!("pulse: symbols exported");
        {
            use std::io::Write;
            info!("pulse: exporting types for ida...");
            let mut ida_file = File::create("hfw_ggrtti.idc").unwrap();
            let mut existing_containers = Vec::<*mut RTTIContainerData>::new();
            let mut existing_pointers = Vec::<*mut RTTIPointerData>::new();
            writeln!(ida_file, "#include <idc.idc>\n\nstatic main() {{").unwrap();
            types.iter().for_each(|rtti| unsafe {
                export_ida_type(
                    *rtti,
                    &mut ida_file,
                    &mut existing_containers,
                    &mut existing_pointers,
                )
                .unwrap();
            });
            writeln!(ida_file, "}}").unwrap();
        }
        info!("pulse: done");
    }
}

unsafe impl Sync for PulsePlugin {}
unsafe impl Send for PulsePlugin {}

define_cauldron_plugin!(PulsePlugin, include_str!("../pulse.cauldron.toml"));

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

fn export_symbols() {
    let groups_offset = find_offset_from(
        "48 8B 3D ? ? ? ? 48 63 05 ? ? ? ? 48 8D 2C C7 48 3B FD 74 45 48 8B",
        3,
    )
    .unwrap()
        - 8usize;
    let groups_ptr = get_module().unwrap().0 + groups_offset;
    let groups: *mut GGArray<*mut ExportedSymbolsGroup> =
        unsafe { std::mem::transmute(groups_ptr) };
    let mut file = File::create("symbols.txt").unwrap();
    for group in unsafe { (*groups).slice() } {
        let group = unsafe { &**group };
        export_symbol(group, &mut file);
    }
}

fn export_symbol<W: std::io::Write>(group: &ExportedSymbolsGroup, file: &mut W) {
    let group_name = unsafe { rtti_name(group.get_rtti()) };
    if group.always_export {
        writeln!(file, "{}: (always exported)", group_name).unwrap();
    } else {
        writeln!(file, "{}:", group_name).unwrap();
    }
    if !group.members.is_empty() {
        writeln!(file, "\tmembers: ({})", group.members.count).unwrap();
        for member in group.members.slice() {
            writeln!(
                file,
                "\t\t{} - {} ({})",
                member.name(),
                member.namespace(),
                member.kind
            )
            .unwrap();
            writeln!(file, "\t\t\tunk20: {:p}", member.unknown_20).unwrap();
            writeln!(file, "\t\t\tunk28: {:p}", member.unknown_28).unwrap();

            writeln!(file, "\t\t\tlanguage: ").unwrap();
            for language in &member.language_info {
                if language.name.is_null() {
                    debug!("nullptr lang: {:?}", language);
                    break;
                }
                writeln!(file, "\t\t\t\t{}:", language.name()).unwrap();
                writeln!(file, "\t\t\t\t\tunk10: {:p}", language.unknown_10).unwrap();
                writeln!(file, "\t\t\t\t\tunk10: {:p}", language.unknown_18).unwrap();
                writeln!(file, "\t\t\t\t\tunk30: {:p}", language.unknown_30).unwrap();
                writeln!(file, "\t\t\t\t\tunk38: {:p}", language.unknown_38).unwrap();
                writeln!(file, "\t\t\t\t\tunk40: {:p}", language.unknown_40).unwrap();
                writeln!(file, "\t\t\t\t\tunk48: {:p}", language.unknown_48).unwrap();
                writeln!(file, "\t\t\t\t\tunk50: {:p}", language.unknown_50).unwrap();
                writeln!(file, "\t\t\t\t\tunk58: {:p}", language.unknown_58).unwrap();
                writeln!(file, "\t\t\t\t\tunk60: {:p}", language.unknown_60).unwrap();
                writeln!(file, "\t\t\t\t\tunk68: {:p}", language.unknown_68).unwrap();

                writeln!(file, "\t\t\t\t\tsignature:").unwrap();
                for part in language.signature.slice() {
                    writeln!(file, "\t\t\t\t\t\t{}:", part.name()).unwrap();
                    writeln!(file, "\t\t\t\t\t\t\tmodifiers: {}", part.modifiers()).unwrap();
                    writeln!(file, "\t\t\t\t\t\t\tunk10: {:p}", part.unknown_10).unwrap();
                    writeln!(file, "\t\t\t\t\t\t\tunk18: {:p}", part.unknown_18).unwrap();
                    writeln!(file, "\t\t\t\t\t\t\tunk20: {}", part.unknown_20).unwrap();
                }
            }
        }
    }
    if !group.dependencies.is_empty() {
        writeln!(file, "\tdependencies: ({})", group.dependencies.count).unwrap();
        for dep in group.dependencies.slice() {
            let name = unsafe { rtti_name(*dep) };
            writeln!(file, "\t\t{} ({})", name, unsafe { (*(*dep)).kind }).unwrap();
        }
    }
}
