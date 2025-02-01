use crate::assert_size;
use crate::types::decima::manual_types::{Array, HashMap};
use crate::types::rtti::RTTI;
use std::ffi::{c_char, c_void};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ExportedSymbolKind {
    Atom,
    Enum,
    Class,
    Struct,
    Typedef,
    Function,
    Variable,
    Container,
    Reference,
    Pointer,
    Unk10,
}
assert_size!(ExportedSymbolKind, 0x1);

impl Display for ExportedSymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ExportedSymbolKind::Atom => "Atom",
            ExportedSymbolKind::Enum => "Enum",
            ExportedSymbolKind::Class => "Class",
            ExportedSymbolKind::Struct => "Struct",
            ExportedSymbolKind::Typedef => "Typedef",
            ExportedSymbolKind::Function => "Function",
            ExportedSymbolKind::Variable => "Variable",
            ExportedSymbolKind::Container => "Container",
            ExportedSymbolKind::Reference => "Reference",
            ExportedSymbolKind::Pointer => "Pointer",
            ExportedSymbolKind::Unk10 => "Unk10",
        })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbolSignature {
    pub name: *const c_char,
    pub modifiers: *const c_char,
    pub r#type: *mut RTTI,
    pub unk18: *mut c_void,
    pub unk20: u8,
}
assert_size!(ExportedSymbolSignature, 0x28);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbolLanguage {
    pub address: *mut c_void,
    pub name: *const c_char,
    pub unk10: *mut c_void,
    pub unk18: *mut c_void,
    pub signature: Array<ExportedSymbolSignature>,
    pub unk30: *mut c_void,
    pub unk38: *mut c_void,
}
assert_size!(ExportedSymbolLanguage, 0x40);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbol {
    pub kind: ExportedSymbolKind,
    pub r#type: *const RTTI,
    pub namespace: *const c_char,
    pub name: *const c_char,
    pub unk20: *mut c_void,
    pub unk28: u8,
    pub language: [ExportedSymbolLanguage; 2],
}
assert_size!(ExportedSymbol, 0xB0);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbolGroup {
    // base@0: RTTIObject
    // pub rtti_object__fn_get_rtti: *mut c_void,
    pub fn_register_symbols: *mut c_void,

    pub export_mask: u32,
    pub namespace: *const c_char,
    pub symbols: Array<ExportedSymbol>,
    pub dependencies: Array<*const RTTI>,
}
assert_size!(ExportedSymbolGroup, 0x38);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbols {
    pub groups: Array<*mut ExportedSymbolGroup>,
    pub dependencies_unk1: Array<*const RTTI>,
    pub dependencies_unk2: Array<*const RTTI>,
    pub all_symbols: HashMap<*mut ExportedSymbol, u32>,
    pub type_symbols: HashMap<*const c_char, *mut ExportedSymbol>,
}
