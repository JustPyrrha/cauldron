use crate::types::{
    as_compound, as_container, as_enum, as_pointer, RTTIAttribute, RTTIBase, RTTICompound,
    RTTIContainer, RTTIContainerData, RTTIEnum, RTTIKind, RTTIMessageHandler,
    RTTIMessageOrderEntry, RTTIPod, RTTIPointer, RTTIPointerData, RTTIPrimitive, RTTIValue, RTTI,
};
use libc::c_char;
use std::ffi::CStr;
use std::mem;

fn ida_rtti_kind_name(rtti: &RTTI) -> &str {
    match rtti.kind {
        RTTIKind::Primitive => "RTTIPrimitive",
        RTTIKind::Pointer => "RTTIPointer",
        RTTIKind::Container => "RTTIContainer",
        RTTIKind::Compound => "RTTICompound",
        RTTIKind::Enum | RTTIKind::EnumFlags => "RTTIEnum",
        RTTIKind::POD => "RTTIPod",

        _ => unreachable!(),
    }
}

unsafe fn from_c_str(cstr: &*const c_char) -> &str {
    CStr::from_ptr(*cstr).to_str().unwrap()
}

fn ida_rtti_type_name(rtti: &RTTI) -> String {
    match rtti.kind {
        RTTIKind::Primitive => unsafe {
            let primitive: &RTTIPrimitive = mem::transmute(rtti);
            from_c_str(&primitive.name).to_string()
        },
        RTTIKind::Pointer => unsafe {
            let pointer: &RTTIPointer = mem::transmute(rtti);
            format!(
                "{}_{}",
                from_c_str(&(*pointer.pointer_type).name),
                ida_rtti_type_name(&*pointer.item_type)
            )
        },
        RTTIKind::Container => unsafe {
            let container: &RTTIContainer = mem::transmute(rtti);
            format!(
                "{}_{}",
                from_c_str(&(*container.container_type).name),
                ida_rtti_type_name(&*container.item_type)
            )
        },
        RTTIKind::Enum | RTTIKind::EnumFlags => unsafe {
            let enum_: &RTTIEnum = mem::transmute(rtti);
            from_c_str(&enum_.name).to_string()
        },
        RTTIKind::Compound => unsafe {
            let compound: &RTTICompound = mem::transmute(rtti);
            from_c_str(&compound.name).to_string()
        },
        RTTIKind::POD => unsafe {
            let pod: &RTTIPod = mem::transmute(rtti);
            format!("POD{}", pod.size)
        },

        _ => unreachable!(),
    }
}

pub unsafe fn export_ida_type<W: std::io::Write>(
    rtti_ptr: *const RTTI,
    file: &mut W,
    existing_containers: &mut Vec<*mut RTTIContainerData>,
    existing_pointers: &mut Vec<*mut RTTIPointerData>,
) -> std::io::Result<()> {
    let rtti = &*rtti_ptr;
    let kind_name = ida_rtti_kind_name(rtti);
    let type_name = ida_rtti_type_name(rtti);

    writeln!(file, "\t// {} {}", kind_name, type_name)?;
    writeln!(file, "\tset_name({:p}, \"RTTI_{}\");", rtti_ptr, type_name)?;
    writeln!(file, "\tapply_type({:p}, \"{}\");", rtti_ptr, type_name)?;

    if let Some(compound) = as_compound(rtti_ptr) {
        let compound = &*compound;
        if compound.num_bases > 0 && !compound.bases.is_null() {
            writeln!(
                file,
                "\tdel_items({:p}, DELIT_SIMPLE, {});",
                compound.bases,
                compound.num_bases as usize * size_of::<RTTIBase>()
            )?;
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIBase[{}]\");",
                compound.bases, compound.num_bases
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sBases\");",
                compound.bases, type_name
            )?;
        }

        if compound.num_attributes > 0 && !compound.attributes.is_null() {
            writeln!(
                file,
                "\tdel_items({:p}, DELIT_SIMPLE, {});",
                compound.attributes,
                compound.num_attributes as usize * size_of::<RTTIAttribute>()
            )?;
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIAttr[{}]\");",
                compound.attributes, compound.num_attributes
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sAttrs\");",
                compound.attributes, type_name
            )?;
        }

        if compound.num_message_handlers > 0 && !compound.message_handlers.is_null() {
            writeln!(
                file,
                "\tdel_items({:p}, DELIT_SIMPLE, {});",
                compound.message_handlers,
                compound.num_message_handlers as usize * size_of::<RTTIMessageHandler>()
            )?;
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIMessageHandler[{}]\");",
                compound.message_handlers, compound.num_message_handlers
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sMessageHandlers\");",
                compound.message_handlers, type_name
            )?;

            compound
                .message_handlers()
                .iter()
                .enumerate()
                .for_each(|(i, handler)| {
                    let message_name = ida_rtti_type_name(&*handler.message);
                    writeln!(
                        file,
                        "\tset_name({:p}, \"{}::On{}{}\");",
                        handler,
                        type_name,
                        message_name,
                        if i == 0 {
                            String::new()
                        } else {
                            format!("{}", i)
                        }
                    )
                    .unwrap();
                    writeln!(
                        file,
                        "\tapply_type({:p}, \"__int64 __fastcall f(void* this, {}* ioMsg)\");",
                        handler.handler, message_name
                    )
                    .unwrap();
                });
        }

        if compound.num_message_order_entries > 0 && !compound.message_order_entries.is_null() {
            writeln!(
                file,
                "\tdel_items({:p}, DELIT_SIMPLE, {});",
                compound.message_order_entries,
                compound.num_message_order_entries as usize * size_of::<RTTIMessageOrderEntry>()
            )?;
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIInheritedMessageHandler[{}]\");",
                compound.message_order_entries, compound.num_message_order_entries
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sInheritedMessageHandlers\");",
                compound.message_order_entries, type_name
            )?;
        }

        if !compound.get_exported_symbols.is_null() {
            writeln!(
                file,
                "\tset_name({:p}, \"{}::GetExportedSymbols\");",
                compound.get_exported_symbols, type_name
            )?;
        }
    }

    if let Some(enum_) = as_enum(rtti_ptr) {
        let enum_ = &*enum_;
        if enum_.num_values > 0 && !enum_.values.is_null() {
            writeln!(
                file,
                "\tdel_items({:p}, DELIT_SIMPLE, {});",
                enum_.values,
                enum_.num_values as usize * size_of::<RTTIValue>()
            )?;
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIValue[{}]\");",
                enum_.values, enum_.num_values
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sValues\");",
                enum_.values, type_name
            )?;
        }
    }

    if let Some(container) = as_container(rtti_ptr) {
        let container = &*container;
        if !existing_containers.contains(&container.container_type) {
            existing_containers.push(container.container_type);
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIContainer::Data\");",
                container.container_type
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sInfo\");",
                container.container_type, type_name
            )?;
        }
    }

    if let Some(pointer) = as_pointer(rtti_ptr) {
        let pointer = &*pointer;
        if !existing_pointers.contains(&pointer.pointer_type) {
            existing_pointers.push(pointer.pointer_type);
            writeln!(
                file,
                "\tapply_type({:p}, \"RTTIPointer::Data\");",
                pointer.pointer_type
            )?;
            writeln!(
                file,
                "\tset_name({:p}, \"{}::sInfo\");",
                pointer.pointer_type, type_name
            )?;
        }
    }

    Ok(())
}
