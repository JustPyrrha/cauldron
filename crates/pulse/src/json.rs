// use json_writer::JSONObjectWriter;
// use libdecima::types::rtti::symbols::{ExportedSymbolGroup, ExportedSymbols};
// use libdecima::types::rtti::{
//     cstr_to_string, rtti_name, RTTI,
// };
// use std::fs::{File, OpenOptions};
// use std::io::Write as _;
//
// /// Export [Decima Workshop](https://github.com/ShadelessFox/decima) compatible json types with compound alignment and size added.
// ///
// /// See: [decima-native](https://github.com/ShadelessFox/decima-native/blob/hrzr-injector/source/Exporter/JsonExporter.cpp)
// pub(crate) fn export_types_json(types: &Vec<*const RTTI>) {
//     let mut json_str = String::new();
//     {
//         let mut json_writer = JSONObjectWriter::new(&mut json_str);
//
//         let mut containers = Vec::new();
//         let mut pointers = Vec::new();
//
//         for ty in types {
//             export_type_json(*ty, &mut json_writer, &mut containers, &mut pointers);
//         }
//     }
//     let mut file = File::create("hfw_ggrtti.json").unwrap();
//     file.write_all(&json_str.as_bytes()).unwrap();
// }
//
// pub(crate) fn export_symbols_json(symbols: *mut ExportedSymbols) {
//     let mut json_str = String::new();
//     unsafe {
//         let mut json_writer = JSONObjectWriter::new(&mut json_str);
//         for group in (*symbols).groups.slice() {
//             export_symbol_json(&**group, &mut json_writer);
//         }
//     }
//     let mut file = File::create("hfw_symbols.json").unwrap();
//     file.write_all(&json_str.as_bytes()).unwrap()
// }
//
// fn export_type_json(
//     ty: *const RTTI,
//     writer: &mut JSONObjectWriter,
//     containers: &mut Vec<String>,
//     pointers: &mut Vec<String>,
// ) {
//     unsafe {
//         let mut f = OpenOptions::new()
//             .write(true)
//             .append(true)
//             .open("pulse.log")
//             .unwrap();
//         if let Some(container) = as_container(ty) {
//             writeln!(f, "{:#?}", (*container)).unwrap();
//             let name = cstr_to_string((*(*container).container_type).type_name);
//             if containers.contains(&name) {
//                 return;
//             } else {
//                 containers.push(name);
//             }
//         }
//
//         if let Some(pointer) = as_pointer(ty) {
//             writeln!(f, "{:#?}", (*pointer)).unwrap();
//             let name = cstr_to_string((*(*pointer).pointer_type).type_name);
//             if pointers.contains(&name) {
//                 return;
//             } else {
//                 pointers.push(name);
//             }
//         }
//
//         let mut obj = writer.object(rtti_name(ty).as_str());
//         obj.value("kind", (*ty).kind.to_string().as_str());
//         if let Some(compound) = as_compound(ty) {
//             writeln!(f, "{:#?}", (*compound)).unwrap();
//             obj.value("version", (*compound).version);
//             obj.value("flags", (*compound).flags);
//             obj.value("size", (*compound).size);
//             obj.value("alignment", (*compound).alignment);
//
//             if !(*compound).message_handlers().is_empty() {
//                 let mut array = obj.array("messages");
//                 for message in (*compound).message_handlers() {
//                     array.value(rtti_name((*message).message).as_str())
//                 }
//                 array.end();
//             }
//
//             if !(*compound).bases().is_empty() {
//                 let mut array = obj.array("bases");
//                 for base in (*compound).bases() {
//                     let mut base_obj = array.object();
//                     base_obj.value("name", cstr_to_string((*base.r#type).type_name).as_str());
//                     base_obj.value("offset", base.offset);
//                     base_obj.end();
//                 }
//                 array.end();
//             }
//
//             if !(*compound).attributes().is_empty() {
//                 let mut array = obj.array("attrs");
//                 for attr in (*compound).attributes() {
//                     let mut attr_obj = array.object();
//                     if attr.r#type.is_null() {
//                         attr_obj.value("category", cstr_to_string(attr.name).as_str());
//                         attr_obj.end();
//                         continue;
//                     }
//
//                     attr_obj.value("name", cstr_to_string(attr.name).as_str());
//                     attr_obj.value("type", rtti_name(attr.r#type).as_str());
//                     attr_obj.value("offset", attr.offset);
//                     attr_obj.value("flags", attr.flags);
//
//                     if !attr.min_value.is_null() {
//                         attr_obj.value("min", cstr_to_string(attr.min_value).as_str());
//                     }
//                     if !attr.max_value.is_null() {
//                         attr_obj.value("max", cstr_to_string(attr.max_value).as_str());
//                     }
//                     if !attr.fn_getter.is_null() && !attr.fn_setter.is_null() {
//                         attr_obj.value("property", true);
//                     }
//                     attr_obj.end();
//                 }
//                 array.end();
//             }
//         } else if let Some(r#enum) = as_enum(ty) {
//             writeln!(f, "{:#?}", (*r#enum)).unwrap();
//             obj.value("size", (*r#enum).size);
//             let mut array = obj.array("values");
//             for value in (*r#enum).values() {
//                 let mut value_obj = array.object();
//                 value_obj.value("value", value.value);
//                 value_obj.value("name", cstr_to_string(value.name).as_str());
//                 if let Some(aliases) = value.aliases() {
//                     let mut array = value_obj.array("alias");
//                     for alias in aliases {
//                         array.value(alias.as_str());
//                     }
//                     array.end();
//                 }
//                 value_obj.end();
//             }
//             array.end();
//         } else if let Some(atom) = as_atom(ty) {
//             writeln!(f, "{:#?}", (*atom)).unwrap();
//             obj.value(
//                 "base_type",
//                 cstr_to_string((*(*atom).parent_type).type_name).as_str(),
//             );
//         }
//         obj.end();
//     }
// }
//
// fn export_symbol_json(group: &ExportedSymbolGroup, writer: &mut JSONObjectWriter) {
//     let mut namespace = writer.object(cstr_to_string(group.namespace).as_str());
//     for symbol in group.symbols.slice() {
//         let mut obj = namespace.object(cstr_to_string(symbol.name).as_str());
//         obj.value("kind", format!("{}", symbol.kind).as_str());
//         obj.value("namespace", cstr_to_string(symbol.namespace).as_str());
//     }
// }
