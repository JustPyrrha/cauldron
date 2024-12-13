use libc::{c_char, c_void};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::mem;
// rtti reversing work by shadeless: https://github.com/ShadelessFox/decima-native/blob/hfw-injector/
// todo: add a proper credits section to readme lol

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RTTIKind {
    Primitive = 0,
    Pointer = 1,
    Container = 2,
    Enum = 3,
    Compound = 4,
    EnumFlags = 5,
    POD = 6,
    EnumBitSet = 7,

    Unknown = u8::MAX,
}

#[derive(Debug)]
#[repr(C, packed(1))]
pub struct RTTI {
    pub id: u32,
    pub kind: RTTIKind,
    pub factory_flags: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIBase {
    pub base: *mut RTTI,
    pub offset: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIAttribute {
    pub base: *mut RTTI,
    pub offset: u16,
    pub flags: u16,
    pub name: *const c_char,
    pub getter: *const c_void,
    pub setter: *const c_void,
    pub min_value: *const c_char,
    pub max_value: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIMessageHandler {
    pub message: *mut RTTI,
    pub handler: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIMessageOrderEntry {
    pub before: u32,
    pub message: *mut RTTI,
    pub compound: *mut RTTI,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTICompound {
    pub base: RTTI,
    pub num_bases: u8,
    pub num_attributes: u8,
    pub num_message_handlers: u8,
    pub num_message_order_entries: u8,
    pub unk_09: u8,
    pub version: u32,
    pub size: u32,
    pub alignment: u16,
    pub flags: u16,
    pub constructor: *mut c_void,
    pub destructor: *mut c_void,
    pub from_string: *mut c_void,
    pub from_string_slice: *mut c_void,
    pub to_string: *mut c_void,
    pub name: *const c_char,
    pub previous_type: *mut RTTI,
    pub next_type: *mut RTTI,
    pub bases: *mut RTTIBase,
    pub attributes: *mut RTTIAttribute,
    pub message_handlers: *mut RTTIMessageHandler,
    pub message_order_entries: *mut RTTIMessageOrderEntry,
    pub get_exported_symbols: *mut c_void,
    pub representation_type: *mut c_void,
    pub unk_88: *mut c_void,
    pub unk_90: *mut c_void,
    pub unk_98: *mut c_void,
    pub on_read_msg_binary: *mut c_void,
    pub vtable_offset: u32,
    pub unk_ac: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIContainerData {
    pub name: *const c_char,
    pub size: u16,
    pub alignment: u8,
    pub unk_0b: [u8; 5],
    pub constructor: *mut c_void,
    pub destructor: *mut c_void,
    pub unk_20: *mut c_void,
    pub unk_28: *mut c_void,
    pub unk_30: *mut c_void,
    pub get_num_items: *mut c_void,
    pub unk_40: *mut c_void,
    pub unk_48: *mut c_void,
    pub unk_50: *mut c_void,
    pub unk_58: *mut c_void,
    pub unk_60: *mut c_void,
    pub unk_68: *mut c_void,
    pub unk_70: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIContainer {
    pub base: RTTI,
    pub unk_06: u8,
    pub item_type: *mut RTTI,
    pub container_type: *mut RTTIContainerData,
    pub name: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIPointerData {
    pub name: *const c_char,
    pub size: u32,
    pub alignment: u8,
    pub unk_0d: [u8; 3],
    pub constructor: *mut c_void,
    pub destructor: *mut c_void,
    pub getter: *mut c_void,
    pub setter: *mut c_void,
    pub copier: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIPointer {
    pub base: RTTI,
    pub unk_06: [u8; 2],
    pub item_type: *mut RTTI,
    pub pointer_type: *mut RTTIPointerData,
    pub name: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIValue {
    pub value: u64,
    pub name: *const c_char,
    pub aliases: [*const c_char; 4],
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIEnum {
    pub base: RTTI,
    pub size: u8,
    pub alignment: u8,
    pub num_values: u16,
    pub unk_0a: [u8; 6],
    pub name: *const c_char,
    pub values: *mut RTTIValue,
    pub representation_type: *mut RTTI,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTTIPrimitive {
    pub base: RTTI,
    pub size: u16,
    pub alignment: u8,
    pub simple: u8,
    pub unk_08: [u8; 6],
    pub name: *const c_char,
    pub base_type: *mut RTTI,
    pub from_string: *mut c_void,
    pub to_string: *mut c_void,
    pub unk_30: *mut c_void,
    pub assign: *mut c_void,
    pub equals: *mut c_void,
    pub constructor: *mut c_void,
    pub destructor: *mut c_void,
    pub swap_endian: *mut c_void,
    pub try_assign: *mut c_void,
    pub get_size_in_memory: *mut c_void,
    pub compare_strings: *mut c_void,
    pub representation_type: *mut RTTI,
}

// macro_rules! assert_offset {
//     ($container:ty, $field:expr, $offset:expr) => {
//         assert_eq!(std::mem::offset_of!($container, $field), $offset)
//     };
// }

// macro assert_size($container:ty, $size:expr) {
//     assert_eq!{ std::mem::size_of::<$container>(), $size, "std::mem::size_of::<{}>() == {}", std::any::type_name::<$container>(), $size }
// }

// todo: need to fix these

// assert_size!(RTTI, 0x6);
// assert_size!(RTTIBase, 0x10);
// assert_size!(RTTIAttribute, 0x38);
// assert_size!(RTTIMessageHandler, 0x10);
// assert_size!(RTTICompound, 0xB0);
// assert_size!(RTTIContainerData, 0x78);
// assert_size!(RTTIPointerData, 0x38);
// assert_size!(RTTIPointer, 0x20);
// assert_size!(RTTIValue, 0x30);
// assert_size!(RTTIEnum, 0x28);
// assert_size!(RTTIAtom, 0x80);

impl From<u8> for RTTIKind {
    fn from(value: u8) -> Self {
        match value {
            0 => RTTIKind::Primitive,
            1 => RTTIKind::Pointer,
            2 => RTTIKind::Container,
            3 => RTTIKind::Enum,
            4 => RTTIKind::Compound,
            5 => RTTIKind::EnumFlags,
            6 => RTTIKind::POD,
            7 => RTTIKind::EnumBitSet,
            _ => RTTIKind::Unknown,
        }
    }
}

impl Display for RTTIKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RTTIKind::Primitive => f.write_str("primitive"),
            RTTIKind::Pointer => f.write_str("reference"),
            RTTIKind::Container => f.write_str("container"),
            RTTIKind::Enum => f.write_str("enum"),
            RTTIKind::Compound => f.write_str("class"),
            RTTIKind::EnumFlags => f.write_str("enum flags"),
            RTTIKind::POD => f.write_str("pod"),
            RTTIKind::EnumBitSet => f.write_str("enum bitset"),
            RTTIKind::Unknown => f.write_str("unknown"),
        }
    }
}

pub unsafe fn as_primitive(rtti: *const RTTI) -> Option<*const RTTIPrimitive> {
    if (*rtti).kind == RTTIKind::Primitive {
        Some(mem::transmute(rtti))
    } else {
        None
    }
}

pub unsafe fn as_pointer(rtti: *const RTTI) -> Option<*const RTTIPointer> {
    if (*rtti).kind == RTTIKind::Pointer {
        Some(mem::transmute(rtti))
    } else {
        None
    }
}

pub unsafe fn as_container(rtti: *const RTTI) -> Option<*const RTTIContainer> {
    if (*rtti).kind == RTTIKind::Container {
        Some(mem::transmute(rtti))
    } else {
        None
    }
}

pub unsafe fn as_enum(rtti: *const RTTI) -> Option<*const RTTIEnum> {
    if (*rtti).kind == RTTIKind::Enum || (*rtti).kind == RTTIKind::EnumFlags {
        Some(mem::transmute(rtti))
    } else {
        None
    }
}

pub unsafe fn as_compound(rtti: *const RTTI) -> Option<*const RTTICompound> {
    if (*rtti).kind == RTTIKind::Compound {
        Some(mem::transmute(rtti))
    } else {
        None
    }
}

pub unsafe fn rtti_name(rtti: *const RTTI) -> String {
    if let Some(compound) = as_compound(rtti) {
        CStr::from_ptr((*compound).name)
            .to_str()
            .unwrap()
            .to_string()
    } else if let Some(_enum) = as_enum(rtti) {
        CStr::from_ptr((*_enum).name).to_str().unwrap().to_string()
    } else if let Some(primitive) = as_pointer(rtti) {
        CStr::from_ptr((*primitive).name)
            .to_str()
            .unwrap()
            .to_string()
    } else if let Some(container) = as_container(rtti) {
        CStr::from_ptr((*container).name)
            .to_str()
            .unwrap()
            .to_string()
    } else if let Some(pointer) = as_pointer(rtti) {
        CStr::from_ptr((*pointer).name)
            .to_str()
            .unwrap()
            .to_string()
    } else {
        String::new()
    }
}

pub unsafe fn rtti_display_name(rtti: *const RTTI) -> String {
    let mut name = String::new();
    if let Some(container) = as_container(rtti) {
        name.push_str(
            CStr::from_ptr((*(*container).container_type).name)
                .to_str()
                .unwrap(),
        );
        name.push('<');
        name.push_str(rtti_display_name((*container).item_type).as_str());
        name.push('>');
    } else if let Some(pointer) = as_pointer(rtti) {
        name.push_str(
            CStr::from_ptr((*(*pointer).pointer_type).name)
                .to_str()
                .unwrap(),
        );
        name.push('<');
        name.push_str(rtti_display_name((*pointer).item_type).as_str());
        name.push('>');
    } else {
        name.push_str(rtti_name(rtti).as_str());
    }

    name
}
