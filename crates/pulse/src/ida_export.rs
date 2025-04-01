use libdecima::types::decima::core::exported_symbols::{
    ExportedSymbolKind, ExportedSymbols, ExportedSymbolsGroup,
};
use libdecima::types::decima::core::rtti::{
    RTTI, RTTICompoundAttribute, RTTICompoundBase, RTTICompoundMessageHandler,
    RTTICompoundMessageOrderEntry, RTTIContainerData, RTTIEnumValue, RTTIKind,
};
use libdecima::util::ReadCStr;
use std::ffi::CStr;
use std::fs::File;
use std::io::Write as _;

pub fn ida_export(types: Vec<&RTTI>) -> anyhow::Result<()> {
    let mut file = File::create("cauldron/plugins/pulse/output/hfw_ida.idc")?;

    writeln!(
        file,
        r#"
#include <idc.idc>

// Check if a function is unique by ensuring it's only referenced by [inType],
// while also allowing references from the ".pdata" and ".rdata" segments.
// Any other references make [inFunction] not unique.
static is_unique_function(inType, inFunction) {{
    auto ref = get_first_dref_to(inFunction);
    while (ref != BADADDR) {{
        auto seg = get_segm_name(ref);
        if (ref != inType && seg != ".pdata" && seg != ".rdata")
            return 0;
        ref = get_next_dref_to(inFunction, ref);
    }}
    return 1;
}}

static main() {{"#
    )?;

    let mut containers = Vec::new();
    for r#type in &types {
        export_type(&mut file, r#type, &mut containers)?;
    }

    for r#type in &types {
        export_type_funcs(&mut file, r#type)?;
    }

    let symbols = ExportedSymbols::get().unwrap();
    for group in symbols.groups.as_slice() {
        let group = unsafe { &*(*group) };
        export_symbols(&mut file, group)?;
    }

    #[cfg(debug_assertions)]
    {
        let mut dump = File::create("cauldron/plugins/pulse/output/dump.txt")?;
        for group in symbols.groups.as_slice() {
            let group = unsafe { &*(*group) };
            dump_symbols(&mut dump, group)?;
        }
    }

    writeln!(file, "}}")?;

    // debugger software breakpoint
    #[cfg(all(debug_assertions, feature = "debug_breakpoints"))]
    unsafe {
        std::arch::asm!("int3")
    };

    Ok(())
}

fn ida_kind_name(kind: &RTTIKind) -> String {
    match kind {
        RTTIKind::Atom => "HZR2::RTTIAtom",
        RTTIKind::Pointer => "HZR2::RTTIPointer",
        RTTIKind::Container => "HZR2::RTTIContainer",
        RTTIKind::Enum | RTTIKind::EnumFlags => "HZR2::RTTIEnum",
        RTTIKind::Compound => "HZR2::RTTICompound",
        RTTIKind::POD => "HZR2::RTTIPod",
        RTTIKind::EnumBitSet => "HZR2::RTTIBitSet",
    }
    .to_string()
}

fn ida_type_symbol_name(rtti: &RTTI) -> String {
    // eg CPtr<Type> -> CPtr__Type
    rtti.get_symbol_name().replace("<", "__").replace(">", "")
}

fn export_type(
    file: &mut File,
    rtti: &RTTI,
    container_types: &mut Vec<*const RTTIContainerData>,
) -> anyhow::Result<()> {
    let kind_str = ida_kind_name(&rtti.kind);
    let type_str = ida_type_symbol_name(&rtti);

    writeln!(file, "\t// {type_str} ({kind_str})")?;
    writeln!(
        file,
        "\tset_name({rtti:p}, \"RTTI_{type_str}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
    )?;
    writeln!(file, "\tapply_type({rtti:p}, \"{kind_str}\");")?;

    if let Some(class) = rtti.as_compound() {
        let bases_len = class.bases_len as usize;
        let bases = class.bases;
        if !bases.is_null() {
            writeln!(
                file,
                "\tdel_items({bases:p}, DELIT_SIMPLE, {});",
                bases_len * size_of::<RTTICompoundBase>()
            )?;
            writeln!(
                file,
                "\tapply_type({bases:p}, \"HZR2::RTTICompound::Base[{bases_len}]\");"
            )?;
            writeln!(
                file,
                "\tset_name({bases:p}, \"{type_str}::bases\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
        }

        let attrs_len = class.attributes_len as usize;
        let attrs = class.attributes;
        if !attrs.is_null() {
            writeln!(
                file,
                "\tdel_items({attrs:p}, DELIT_SIMPLE, {});",
                attrs_len * size_of::<RTTICompoundAttribute>()
            )?;
            writeln!(
                file,
                "\tapply_type({attrs:p}, \"HZR2::RTTICompound::Attribute[{attrs_len}]\");"
            )?;
            writeln!(
                file,
                "\tset_name({attrs:p}, \"{type_str}::attributes\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
        }

        let msg_handlers_len = class.message_handlers_len as usize;
        let msg_handlers = class.message_handlers;
        if !msg_handlers.is_null() {
            writeln!(
                file,
                "\tdel_items({msg_handlers:p}, DELIT_SIMPLE, {});",
                msg_handlers_len * size_of::<RTTICompoundMessageHandler>()
            )?;
            writeln!(
                file,
                "\tapply_type({msg_handlers:p}, \"HZR2::RTTICompound::MessageHandler[{msg_handlers_len}]\");"
            )?;
            writeln!(
                file,
                "\tset_name({msg_handlers:p}, \"{type_str}::message_handlers\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
        }

        let msg_order_entries_len = class.message_order_entries_len as usize;
        let msg_order_entries = class.message_order_entries;
        if !msg_order_entries.is_null() {
            writeln!(
                file,
                "\tdel_items({msg_order_entries:p}, DELIT_SIMPLE, {});",
                msg_order_entries_len * size_of::<RTTICompoundMessageOrderEntry>()
            )?;
            writeln!(
                file,
                "\tapply_type({msg_order_entries:p}, \"HZR2::RTTICompound::MessageOrderEntry[{msg_order_entries_len}]\");"
            )?;
            writeln!(
                file,
                "\tset_name({msg_order_entries:p}, \"{type_str}::message_order_entries\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
        }
    }

    if let Some(r#enum) = rtti.as_enum() {
        let values = r#enum.values;
        let values_len = r#enum.values_len as usize;
        if !values.is_null() {
            writeln!(
                file,
                "\tdel_items({values:p}, DELIT_SIMPLE, {});",
                values_len * size_of::<RTTIEnumValue>()
            )?;
            writeln!(
                file,
                "\tapply_type({values:p}, \"HZR2::RTTIEnum::Value[{values_len}]\");"
            )?;
            writeln!(
                file,
                "\tset_name({values:p}, \"{type_str}::message_order_entries\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
        }
    }

    if let Some(container) = rtti.as_container()
        && unsafe { container_types.contains(&std::mem::transmute(container.container_type)) }
    {
        unsafe { container_types.push(std::mem::transmute(container.container_type)) };
        if container.base.kind == RTTIKind::Pointer {
            writeln!(
                file,
                "\tapply_type({container:p}, \"HZR2::RTTIContainer::PointerData\");"
            )?;
        } else {
            writeln!(
                file,
                "\tapply_type({container:p}, \"HZR2::RTTIContainer::ContainerData\");"
            )?;
        }
        writeln!(
            file,
            "\tset_name({container:p}, \"{type_str}::info\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
        )?;
    }

    Ok(())
}

fn export_type_funcs(file: &mut File, rtti: &RTTI) -> anyhow::Result<()> {
    let type_str = ida_type_symbol_name(&rtti);

    if let Some(class) = rtti.as_compound() {
        if !class.fn_constructor.is_null() {
            let ctor = class.fn_constructor as *mut _;
            writeln!(file, "\tif (is_unique_function({class:p}, {ctor:p})) {{")?;
            writeln!(
                file,
                "\t\tapply_type({ctor:p}, \"void * __fastcall f(RTTI *inType, void *inObject)\");"
            )?;
            writeln!(
                file,
                "\t\tset_name({ctor:p}, \"{type_str}::Constructor\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
            writeln!(file, "\t}}")?;
        }

        if !class.fn_destructor.is_null() {
            let dtor = class.fn_destructor as *mut _;
            writeln!(file, "\tif (is_unique_function({class:p}, {dtor:p})) {{")?;
            writeln!(
                file,
                "\t\tapply_type({dtor:p}, \"void * __fastcall f(RTTI *inType, void *inObject)\");"
            )?;
            writeln!(
                file,
                "\t\tset_name({dtor:p}, \"{type_str}::Destructor\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
            writeln!(file, "\t}}")?;
        }

        if !class.fn_get_symbol_group.is_null() {
            let symbols = class.fn_get_symbol_group as *mut _;
            writeln!(file, "\tif (is_unique_function({class:p}, {symbols:p})) {{")?;
            writeln!(
                file,
                "\t\tapply_type({symbols:p}, \"const RTTI *__fastcall f()\");"
            )?;
            writeln!(
                file,
                "\t\tset_name({symbols:p}, \"{type_str}::GetExportedSymbols\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
            writeln!(file, "\t}}")?;
        }

        for attr in class.attributes() {
            if attr.is_group() {
                continue;
            }

            let attrs = class.attributes;
            let attr_name = unsafe { CStr::from_ptr(attr.name).to_string_lossy().to_string() };

            if !attr.fn_get.is_null() {
                let getter = attr.fn_get as *mut _;
                writeln!(file, "\tif (is_unique_function({attrs:p}, {getter:p})) {{")?;
                writeln!(
                    file,
                    "\t\tapply_type({getter:p}, \"void *__fastcall f(void *this)\");"
                )?;
                writeln!(
                    file,
                    "\t\tset_name({getter:p}, \"{type_str}::Get{attr_name}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
                )?;
                writeln!(file, "\t}}")?;
            }

            if !attr.fn_set.is_null() {
                let setter = attr.fn_set as *mut _;
                writeln!(file, "\tif (is_unique_function({attrs:p}, {setter:p})) {{")?;
                writeln!(
                    file,
                    "\t\tapply_type({setter:p}, \"void __fastcall f(void *this, void *inValue)\");"
                )?;
                writeln!(
                    file,
                    "\t\tset_name({setter:p}, \"{type_str}::Set{attr_name}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
                )?;
                writeln!(file, "\t}}")?;
            }
        }

        for handler in class.message_handlers() {
            let msg_name = ida_type_symbol_name(unsafe { &*handler.message });
            let handlers = class.message_handlers;
            writeln!(
                file,
                "\tif (is_unique_function({handlers:p}, {handler:p})) {{"
            )?;
            writeln!(
                file,
                "\t\tapply_type({handler:p}, \"void *__fastcall f(void *this, {msg_name} *ioMsg)\");"
            )?;
            writeln!(
                file,
                "\t\tset_name({handler:p}, \"{type_str}::On{msg_name}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
            )?;
            writeln!(file, "\t}}")?;
        }
    }

    Ok(())
}

fn export_symbols(file: &mut File, group: &ExportedSymbolsGroup) -> anyhow::Result<()> {
    for symbol in group.symbols.as_slice() {
        if symbol.kind != ExportedSymbolKind::Variable
            && symbol.kind != ExportedSymbolKind::Function
        {
            continue;
        }

        for language in &symbol.language {
            if language.name.is_null() {
                break;
            }

            let addr = language.address;
            let name = unsafe { CStr::from_ptr(language.name).to_string_lossy().to_string() };
            if !symbol.namespace.is_null() {
                writeln!(
                    file,
                    "\tset_name({addr:p}, \"{}::{name}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);",
                    symbol.namespace().unwrap()
                )?;
            } else {
                writeln!(
                    file,
                    "\tset_name({addr:p}, \"{name}\", SN_FORCE | SN_DELTAIL | SN_NOWARN);"
                )?;
            }

            if symbol.kind == ExportedSymbolKind::Function {
                let sigs = &language.signature;
                let sig = &sigs.as_slice()[0];

                let sig_name =
                    unsafe { CStr::from_ptr(sig.type_name).to_string_lossy().to_string() };
                let sig_modifiers = if !sig.modifiers.is_null() {
                    unsafe { CStr::from_ptr(sig.modifiers).to_string_lossy().to_string() }
                } else {
                    String::new()
                };

                write!(
                    file,
                    "\tset_func_cmt({addr:p}, \"{sig_name}{sig_modifiers} "
                )?;
                if let Some(namespace) = symbol.namespace() {
                    write!(file, "{namespace}::")?;
                }
                write!(file, "{name}(")?;
                for (idx, sig) in sigs.as_slice().iter().enumerate() {
                    if idx == 0 {
                        continue;
                    }
                    if idx > 1 {
                        write!(file, ", ")?;
                    }
                    let type_name =
                        unsafe { CStr::from_ptr(sig.type_name).to_string_lossy().to_string() };
                    let modifiers =
                        unsafe { CStr::from_ptr(sig.modifiers).to_string_lossy().to_string() };
                    let name = if let Some(name) = sig.name.read_optional_string() {
                        format!(" {}", name)
                    } else {
                        String::new()
                    };
                    write!(file, "{type_name}{modifiers}{name}")?;
                }

                writeln!(file, ");\", 0);")?;
            }
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn dump_symbols(file: &mut File, group: &ExportedSymbolsGroup) -> anyhow::Result<()> {
    for symbol in group.symbols.as_slice() {
        for language in &symbol.language {
            if language.name.is_null() {
                break;
            }

            let namespace = if let Some(str) = symbol.namespace() {
                format!("{str}::")
            } else {
                String::new()
            };
            let name = unsafe { CStr::from_ptr(language.name).to_string_lossy().to_string() };
            let ty = &symbol.kind;
            let addr = language.address;
            writeln!(file, "{ty} {namespace}{name} {addr:p} {{")?;

            for sig in language.signature.as_slice() {
                let type_name = sig
                    .type_name
                    .read_optional_string()
                    .unwrap_or(String::new());
                let modifiers = sig
                    .modifiers
                    .read_optional_string()
                    .unwrap_or(String::new());
                let name = if let Some(name) = sig.name.read_optional_string() {
                    name
                } else {
                    String::new()
                };
                let flags = &sig.flags;
                writeln!(file, "\t{type_name}, {modifiers}, {name}, {flags:?}")?;
            }

            writeln!(file, "}}")?;
        }
    }

    Ok(())
}
