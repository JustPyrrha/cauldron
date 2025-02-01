// use change_case::{pascal_case, snake_case};
// use chrono::prelude::Utc;
// use codegen::{Block, Scope, Type};
// use serde::{Deserialize, Serialize};
// use std::cmp::{Ordering, PartialEq};
// use std::collections::HashMap;
// use std::fs::File;
// use std::io::Write;
// use std::{env, fs};
//
// fn main() {
//     // println!("cargo:rerun-if-changed=build.rs");
//     println!("cargo:rerun-if-changed=data/*.json");
//
//     // gen_bindings("hfw");
// }
//
// // todo: maybe merge with above and #[serde(skip)]?
// #[derive(Debug, Clone, PartialEq, Eq)]
// enum NamedRTTIType {
//     Primitive {
//         name: String,
//         base_type: String,
//         size: u32,
//     },
//     Enum {
//         name: String,
//         size: u32,
//         values: Vec<EnumValue>,
//     },
//     Class {
//         name: String,
//         version: u32,
//         flags: u32,
//         size: u32,
//         alignment: u32,
//         messages: Option<Vec<String>>,
//         bases: Option<Vec<Base>>,
//         attrs: Option<Vec<Attrs>>,
//     },
//     Container {
//         name: String,
//         size: u32,
//         alignment: u32,
//     },
//     Pointer {
//         name: String,
//         size: u32,
//         alignment: u32,
//     },
//     KnownPrimitive {
//         name: String,
//         size: u32,
//     },
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct RTTIFile {
//     #[serde(flatten)]
//     types: HashMap<String, RTTIType>,
// }
//
// #[allow(non_camel_case_types)]
// #[derive(Debug, PartialEq, Eq)]
// enum KnownPrimitives {
//     bool,
//     int,
//     int8,
//     int16,
//     int32,
//     int64,
//     intptr,
//     uint,
//     uint8,
//     uint16,
//     uint32,
//     uint64,
//     uint128,
//     uintptr,
//     float,
//     double,
//     HalfFloat,
//     tchar,
//     wchar,
//     ucs4,
//
//     Unknown,
// }
//
// impl From<&String> for KnownPrimitives {
//     fn from(value: &String) -> Self {
//         match value.as_str() {
//             "bool" => KnownPrimitives::bool,
//             "int" => KnownPrimitives::int,
//             "int8" => KnownPrimitives::int8,
//             "int16" => KnownPrimitives::int16,
//             "int32" => KnownPrimitives::int32,
//             "int64" => KnownPrimitives::int64,
//             "intptr" => KnownPrimitives::intptr,
//             "uint" => KnownPrimitives::uint,
//             "uint8" => KnownPrimitives::uint8,
//             "uint16" => KnownPrimitives::uint16,
//             "uint32" => KnownPrimitives::uint32,
//             "uint64" => KnownPrimitives::uint64,
//             "uint128" => KnownPrimitives::uint128,
//             "uintptr" => KnownPrimitives::uintptr,
//             "float" => KnownPrimitives::float,
//             "double" => KnownPrimitives::double,
//             "HalfFloat" => KnownPrimitives::HalfFloat,
//             "tchar" => KnownPrimitives::tchar,
//             "wchar" => KnownPrimitives::wchar,
//             "ucs4" => KnownPrimitives::ucs4,
//             &_ => KnownPrimitives::Unknown,
//         }
//     }
// }
//
// impl KnownPrimitives {
//     pub fn to_string(&self) -> String {
//         String::from(match self {
//             KnownPrimitives::bool => "bool",
//             KnownPrimitives::int => "i32",
//             KnownPrimitives::int8 => "i8",
//             KnownPrimitives::int16 => "i16",
//             KnownPrimitives::int32 => "i32",
//             KnownPrimitives::int64 => "i64",
//             KnownPrimitives::intptr => "i64",
//             KnownPrimitives::uint => "u32",
//             KnownPrimitives::uint8 => "u8",
//             KnownPrimitives::uint16 => "u16",
//             KnownPrimitives::uint32 => "u32",
//             KnownPrimitives::uint64 => "u64",
//             KnownPrimitives::uint128 => "u128",
//             KnownPrimitives::uintptr => "u64",
//             KnownPrimitives::float => "f32",
//             KnownPrimitives::double => "f64",
//             KnownPrimitives::HalfFloat => "f32", // todo: replace with ::half::f16
//             KnownPrimitives::tchar => "char",
//             KnownPrimitives::wchar => "char",
//             KnownPrimitives::ucs4 => "u32",
//             _ => unreachable!(),
//         })
//     }
//     pub fn to_type(&self) -> Type {
//         Type::new(self.to_string().as_str())
//     }
// }
//
// // fn gen_bindings(game_id: &str) {
// //     let manual_types = vec![
// //         // Containers
// //         String::from("Array"),
// //         // Pointers
// //         String::from("Ref"),
// //         String::from("StreamingRef"),
// //         String::from("UUIDRef"),
// //         String::from("WeakPtr"),
// //         String::from("cptr"),
// //     ];
// //
// //     let input: String = fs::read_to_string(format!("data/{}.json", game_id)).unwrap();
// //     let input: RTTIFile = serde_json::from_str(input.as_str()).unwrap();
// //
// //     let mut known_sizes: HashMap<String, u32> = HashMap::new();
// //
// //     let mut sorted_types: Vec<NamedRTTIType> = input
// //         .types
// //         .iter()
// //         .map(|(k, v)| match v {
// //             RTTIType::Primitive { base_type, size } => NamedRTTIType::Primitive {
// //                 name: k.clone(),
// //                 base_type: base_type.clone(),
// //                 size: size.clone(),
// //             },
// //             RTTIType::Enum { size, values } => NamedRTTIType::Enum {
// //                 name: k.clone(),
// //                 size: size.clone(),
// //                 values: values.clone(),
// //             },
// //             RTTIType::Class {
// //                 version,
// //                 flags,
// //                 size,
// //                 alignment,
// //                 messages,
// //                 bases,
// //                 attrs,
// //             } => NamedRTTIType::Class {
// //                 name: k.clone(),
// //                 version: version.clone(),
// //                 flags: flags.clone(),
// //                 size: size.clone(),
// //                 alignment: alignment.clone(),
// //                 messages: messages.clone(),
// //                 bases: bases.clone(),
// //                 attrs: attrs.clone(),
// //             },
// //             RTTIType::Container { size, alignment } => NamedRTTIType::Container {
// //                 name: k.clone(),
// //                 size: size.clone(),
// //                 alignment: alignment.clone(),
// //             },
// //             RTTIType::Pointer { size, alignment } => NamedRTTIType::Pointer {
// //                 name: k.clone(),
// //                 size: size.clone(),
// //                 alignment: alignment.clone(),
// //             },
// //         })
// //         .collect();
// //
// //     // 0. known primitives
// //     // 1. primitives
// //     // 2. enums
// //     // 3. containers
// //     // 4. pointers
// //     // 5. compounds
// //     //    - sorted by base, attr, and message dependencies
// //     sorted_types.sort_by(|first, second| {
// //         let first_name = match first {
// //             NamedRTTIType::Primitive { name, .. } => name.clone(),
// //             NamedRTTIType::Enum { name, .. } => name.clone(),
// //             NamedRTTIType::Class { name, .. } => name.clone(),
// //             NamedRTTIType::Container { name, .. } => name.clone(),
// //             NamedRTTIType::Pointer { name, .. } => name.clone(),
// //             NamedRTTIType::KnownPrimitive { name, .. } => name.clone(),
// //         }
// //         .to_lowercase();
// //
// //         let second_name = match second {
// //             NamedRTTIType::Primitive { name, .. } => name.clone(),
// //             NamedRTTIType::Enum { name, .. } => name.clone(),
// //             NamedRTTIType::Class { name, .. } => name.clone(),
// //             NamedRTTIType::Container { name, .. } => name.clone(),
// //             NamedRTTIType::Pointer { name, .. } => name.clone(),
// //             NamedRTTIType::KnownPrimitive { name, .. } => name.clone(),
// //         }
// //         .to_lowercase();
// //
// //         if (matches!(first, NamedRTTIType::KnownPrimitive { .. })
// //             && matches!(second, NamedRTTIType::KnownPrimitive { .. }))
// //             || (matches!(first, NamedRTTIType::Primitive { .. })
// //                 && matches!(second, NamedRTTIType::Primitive { .. }))
// //             || (matches!(first, NamedRTTIType::Enum { .. })
// //                 && matches!(second, NamedRTTIType::Enum { .. }))
// //             || (matches!(first, NamedRTTIType::Pointer { .. })
// //                 && matches!(second, NamedRTTIType::Pointer { .. }))
// //             || (matches!(first, NamedRTTIType::Container { .. })
// //                 && matches!(second, NamedRTTIType::Container { .. }))
// //         {
// //             first_name.cmp(&second_name)
// //         } else if (matches!(first, NamedRTTIType::KnownPrimitive { .. })
// //             && matches!(second, NamedRTTIType::Primitive { .. }))
// //             || (matches!(first, NamedRTTIType::Primitive { .. })
// //                 && matches!(second, NamedRTTIType::Enum { .. }))
// //             || (matches!(first, NamedRTTIType::Enum { .. })
// //                 && matches!(second, NamedRTTIType::Container { .. }))
// //             || (matches!(first, NamedRTTIType::Container { .. })
// //                 && matches!(second, NamedRTTIType::Pointer { .. }))
// //             || (matches!(first, NamedRTTIType::Pointer { .. })
// //                 && matches!(second, NamedRTTIType::Class { .. }))
// //         {
// //             Ordering::Greater
// //         } else if (matches!(first, NamedRTTIType::Primitive { .. })
// //             && matches!(second, NamedRTTIType::KnownPrimitive { .. }))
// //             || (matches!(first, NamedRTTIType::Enum { .. })
// //                 && matches!(second, NamedRTTIType::Primitive { .. }))
// //             || (matches!(first, NamedRTTIType::Container { .. })
// //                 && matches!(second, NamedRTTIType::Enum { .. }))
// //             || (matches!(first, NamedRTTIType::Pointer { .. })
// //                 && matches!(second, NamedRTTIType::Container { .. }))
// //             || (matches!(first, NamedRTTIType::Class { .. })
// //                 && matches!(second, NamedRTTIType::Pointer { .. }))
// //         {
// //             Ordering::Less
// //         } else if (matches!(first, NamedRTTIType::Class { .. })
// //             && matches!(second, NamedRTTIType::Class { .. }))
// //         {
// //             let NamedRTTIType::Class {
// //                 name,
// //                 attrs,
// //                 bases,
// //                 messages,
// //                 ..
// //             } = first
// //             else {
// //                 unreachable!()
// //             };
// //             let (a_name, a_attrs, a_bases, a_messages) =
// //                 (name.clone(), attrs.clone(), bases.clone(), messages.clone());
// //             let NamedRTTIType::Class {
// //                 name,
// //                 attrs,
// //                 bases,
// //                 messages,
// //                 ..
// //             } = second
// //             else {
// //                 unreachable!()
// //             };
// //             let (b_name, b_attrs, b_bases, b_messages) =
// //                 (name.clone(), attrs.clone(), bases.clone(), messages.clone());
// //
// //             if b_attrs.is_some_and(|attr| {
// //                 attr.iter().any(|attr| match attr {
// //                     Attrs::Category(_) => false,
// //                     Attrs::Attr(attr) => attr.name.split("<").collect::<Vec<_>>()[0] == a_name,
// //                 })
// //             }) || b_bases.is_some_and(|bases| bases.iter().any(|base| base.name == a_name))
// //                 || b_messages.is_some_and(|msgs| msgs.contains(&a_name))
// //             {
// //                 Ordering::Less
// //             } else if a_attrs.is_some_and(|attr| {
// //                 attr.iter().any(|attr| match attr {
// //                     Attrs::Category(_) => false,
// //                     Attrs::Attr(attr) => attr.name.split("<").collect::<Vec<_>>()[0] == b_name,
// //                 })
// //             }) || a_bases
// //                 .is_some_and(|bases| bases.iter().any(|base| base.name == b_name))
// //                 || a_messages.is_some_and(|msgs| msgs.contains(&b_name))
// //             {
// //                 Ordering::Greater
// //             } else {
// //                 // todo: see why using string::cmp causes error "does not implement a total order"
// //                 Ordering::Equal
// //             }
// //         } else {
// //             // todo: same as above
// //             Ordering::Equal
// //         }
// //     });
// //
// //     for r#type in &sorted_types {
// //         match r#type {
// //             NamedRTTIType::KnownPrimitive { name, size } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //             NamedRTTIType::Primitive { name, size, .. } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //             NamedRTTIType::Enum { name, size, .. } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //             NamedRTTIType::Class { name, size, .. } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //             NamedRTTIType::Container { name, size, .. } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //             NamedRTTIType::Pointer { name, size, .. } => {
// //                 known_sizes.insert(name.clone(), size.clone());
// //             }
// //         }
// //     }
// //
// //     sorted_types = sorted_types
// //         .into_iter()
// //         .filter(|ty| {
// //             !manual_types.contains(match ty {
// //                 NamedRTTIType::Primitive { name, .. } => &name,
// //                 NamedRTTIType::Enum { name, .. } => &name,
// //                 NamedRTTIType::Class { name, .. } => &name,
// //                 NamedRTTIType::Container { name, .. } => &name,
// //                 NamedRTTIType::Pointer { name, .. } => &name,
// //                 NamedRTTIType::KnownPrimitive { name, .. } => &name,
// //             })
// //         })
// //         .collect::<Vec<_>>();
// //
// //     let mut scope = Scope::new();
// //     scope.import("crate", "assert_size");
// //     scope.import("crate", "assert_offset");
// //
// //     for r#type in sorted_types {
// //         match r#type {
// //             NamedRTTIType::KnownPrimitive { name, size } => {}
// //             NamedRTTIType::Primitive {
// //                 name,
// //                 base_type,
// //                 size,
// //             } => {
// //                 scope
// //                     .new_type_alias(
// //                         &name,
// //                         if KnownPrimitives::from(&name) != KnownPrimitives::Unknown {
// //                             KnownPrimitives::from(&name).to_string()
// //                         } else {
// //                             base_type.clone()
// //                         },
// //                     )
// //                     .vis("pub");
// //                 scope.raw(format!("assert_size!({}, {});", &name, size));
// //             }
// //             NamedRTTIType::Enum { name, size, values } => {
// //                 let size_type = if size == 1 {
// //                     "u8"
// //                 } else if size == 2 {
// //                     "u16"
// //                 } else {
// //                     "u32"
// //                 };
// //                 let mut r#enum = scope
// //                     .new_enum(&name)
// //                     .derive("Debug")
// //                     .derive("Clone")
// //                     .derive("PartialEq")
// //                     .derive("Eq")
// //                     .repr(size_type);
// //
// //                 let mut known_values: HashMap<u32, String> = HashMap::new();
// //                 let mut known_aliases: HashMap<String, String> = HashMap::new();
// //
// //                 for value in &values {
// //                     if known_values.contains_key(&value.value) {
// //                         known_aliases.insert(
// //                             fix_variant(&value.name),
// //                             known_values.get(&value.value).unwrap().to_string(),
// //                         );
// //                     } else {
// //                         known_values.insert(value.value.clone(), fix_variant(&value.name));
// //                         r#enum.new_variant(fix_variant(&&value.name));
// //                     }
// //                     if let Some(aliases) = &value.alias {
// //                         for alias in aliases {
// //                             known_aliases.insert(alias.clone(), fix_variant(&value.name));
// //                         }
// //                     }
// //                 }
// //
// //                 scope.raw(format!("assert_size!({}, {});", &name, size).as_str());
// //
// //                 let mut into = scope
// //                     .new_impl(&name)
// //                     .impl_trait(format!("Into<{}>", size_type))
// //                     .new_fn("into")
// //                     .arg_self()
// //                     .ret(size_type);
// //
// //                 let mut block = Block::new("match self");
// //                 for value in &values {
// //                     block.line(format!(
// //                         "{}::{} => {},",
// //                         &name,
// //                         fix_variant(&value.name),
// //                         &value.value
// //                     ));
// //                 }
// //                 into.push_block(block);
// //
// //                 let mut try_from = scope
// //                     .new_impl(&name)
// //                     .impl_trait(format!("TryFrom<{}>", size_type))
// //                     .associate_type("Error", "()")
// //                     .new_fn("try_from")
// //                     .arg("value", size_type)
// //                     .ret("Result<Self, Self::Error>");
// //                 let mut block = Block::new("match value");
// //                 for value in &values {
// //                     block.line(format!(
// //                         "{} => Ok({}::{}),",
// //                         &value.value,
// //                         &name,
// //                         fix_variant(&value.name)
// //                     ));
// //                 }
// //                 block.line("_ => Err(()),");
// //                 try_from.push_block(block);
// //
// //                 if !known_aliases.is_empty() {
// //                     let mut alias_impl = scope.new_impl(&name);
// //                     for (alias, value) in known_aliases {
// //                         alias_impl.associate_const(
// //                             fix_variant(&alias),
// //                             &name,
// //                             format!("{}::{}", &name, value).as_str(),
// //                             "pub",
// //                         );
// //                     }
// //                 }
// //             }
// //             NamedRTTIType::Class {
// //                 name,
// //                 version,
// //                 flags,
// //                 size,
// //                 alignment,
// //                 messages,
// //                 bases,
// //                 attrs,
// //             } => {
// //                 let mut class = scope
// //                     .new_struct(&name)
// //                     .vis("pub")
// //                     .derive("Debug")
// //                     .repr(format!("C, align({})", alignment).as_str());
// //                 let mut asserts = String::from(format!("assert_size!({}, {});", &name, &size));
// //                 let mut current_offset = 0u32;
// //                 println!("{}", &name);
// //
// //                 if bases.is_some() {
// //                     for base in &bases.unwrap() {
// //                         if current_offset != base.offset {
// //                             class
// //                                 .new_field(
// //                                     format!("pad_{:x}", current_offset).as_str(),
// //                                     format!("[u8;{}]", base.offset - current_offset),
// //                                 )
// //                                 .vis("pub");
// //                             current_offset = base.offset - 1;
// //                         }
// //                         let base_name = fix_field(&format!(
// //                             "base{}",
// //                             if base.offset == 0 {
// //                                 String::new()
// //                             } else {
// //                                 format!("_{:x}", &base.offset)
// //                             }
// //                         ));
// //                         class
// //                             .new_field(base_name.clone(), &base.name)
// //                             .vis("pub")
// //                             .doc(format!("offset: {}", &base.offset));
// //
// //                         current_offset += known_sizes.get(&base.name).unwrap();
// //                         asserts.push_str(
// //                             format!(
// //                                 "\nassert_offset!({}, {}, {});",
// //                                 &name, &base_name, &base.offset
// //                             )
// //                             .as_str(),
// //                         );
// //                     }
// //                 }
// //
// //                 if attrs.is_some() {
// //                     let mut filtered = Vec::new();
// //                     for attr in attrs.unwrap() {
// //                         match attr {
// //                             Attrs::Category(_) => {} // todo: ignore for now, might be nice to have some tagging of categories later
// //                             Attrs::Attr(attr) => {
// //                                 if !attr.property.unwrap_or(false) {
// //                                     // todo: handle properties
// //                                     filtered.push(attr);
// //                                 }
// //                             }
// //                         }
// //                     }
// //
// //                     filtered.sort_by_key(|a| a.offset);
// //
// //                     for attr in filtered {
// //                         println!("{}, {}", attr.offset, current_offset);
// //                         if current_offset != attr.offset {
// //                             class
// //                                 .new_field(
// //                                     format!("pad_{:x}", current_offset).as_str(),
// //                                     format!("[u8;{}]", attr.offset - current_offset),
// //                                 )
// //                                 .vis("pub");
// //                             current_offset = attr.offset - 1;
// //                         }
// //                         class
// //                             .new_field(fix_field(&snake_case(&attr.name)), &attr.r#type)
// //                             .vis("pub")
// //                             .doc(format!("offset: {}, flags: {}", attr.offset, attr.flags));
// //
// //                         current_offset += known_sizes
// //                             .get(&attr.r#type.split("<").collect::<Vec<_>>()[0].to_string())
// //                             .unwrap();
// //                         asserts.push_str(
// //                             format!(
// //                                 "\nassert_offset!({}, {}, {});",
// //                                 &name,
// //                                 fix_field(&attr.name),
// //                                 &attr.offset
// //                             )
// //                             .as_str(),
// //                         );
// //                     }
// //                 }
// //
// //                 scope.raw(asserts);
// //             }
// //             NamedRTTIType::Container {
// //                 name,
// //                 size,
// //                 alignment,
// //             } => {
// //                 let mut container = scope
// //                     .new_struct(&name)
// //                     .repr(format!("C, align({})", alignment).as_str())
// //                     .generic("T")
// //                     .derive("Debug")
// //                     .derive("Clone");
// //                 container
// //                     .new_field("unk", format!("[u8;{}]", size))
// //                     .vis("pub");
// //                 container
// //                     .new_field("marker", "std::marker::PhantomData<T>")
// //                     .vis("pub");
// //                 scope.raw(format!(
// //                     "assert_size!({}<dyn std::any::Any>, {});",
// //                     name, size
// //                 ));
// //             }
// //             NamedRTTIType::Pointer { .. } => { /* manually implemented */ }
// //         }
// //     }
// //
// //     let mut file =
// //         File::create(format!("{}/{}.rs", env::var("OUT_DIR").unwrap(), game_id)).unwrap();
// //     writeln!(file, "/// GENERATED FILE. DO NOT EDIT.").unwrap();
// //     writeln!(file, "/// ").unwrap();
// //     writeln!(
// //         file,
// //         "/// Generated on {} by {}",
// //         Utc::now().format("%+"),
// //         "https://github.com/JustPyrrha/cauldron"
// //     )
// //     .unwrap();
// //     writeln!(file, "").unwrap();
// //     writeln!(file, "").unwrap();
// //     write!(file, "{}", scope.to_string()).unwrap();
// //     writeln!(file, "").unwrap();
// // }
//
// fn fix_variant(name: &String) -> String {
//     pascal_case(
//         match name.as_str() {
//             " == " => String::from("eq"),
//             " != " => String::from("ne"),
//             "" => String::from("blank"),
//             &_ => {
//                 if let Ok(_) = syn::parse_str::<syn::Variant>(name) {
//                     name.clone()
//                 } else {
//                     format!("r#{}", name)
//                 }
//             }
//         }
//         .as_str(),
//     )
// }
//
// fn fix_field(name: &String) -> String {
//     snake_case(
//         match name.as_str() {
//             &_ => {
//                 if let Ok(_) = syn::parse_str::<syn::Ident>(name) {
//                     name.clone()
//                 } else {
//                     format!("r#{}", name)
//                 }
//             }
//         }
//         .as_str(),
//     )
// }

fn main() {} // todo: migrate this to a manually triggered generator, "cargo xtask gen" maybe?
