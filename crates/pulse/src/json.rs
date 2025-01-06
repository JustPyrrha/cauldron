use std::io::Write as _;

use crate::types::{as_compound, as_container, as_enum, as_pointer, as_primitive, rtti_name, RTTI};
use json_writer::JSONObjectWriter;
use libc::c_char;
use std::ffi::CStr;
use std::fs::File;

/// Export [Decima Workshop](https://github.com/ShadelessFox/decima) compatible json types.
///
/// See: [decima-native](https://github.com/ShadelessFox/decima-native/blob/hrzr-injector/source/Exporter/JsonExporter.cpp)
pub(crate) fn export_types_json(types: &Vec<*const RTTI>) {
    let mut json_str = String::new();
    {
        let mut json_writer = JSONObjectWriter::new(&mut json_str);

        let mut containers = Vec::new();
        let mut pointers = Vec::new();

        for ty in types {
            export_type_json(*ty, &mut json_writer, &mut containers, &mut pointers);
        }
    }
    let mut file = File::create("hfw_rtti.json").unwrap();
    file.write_all(&json_str.as_bytes()).unwrap();
}

fn export_type_json(
    ty: *const RTTI,
    writer: &mut JSONObjectWriter,
    containers: &mut Vec<String>,
    pointers: &mut Vec<String>,
) {
    unsafe {
        if let Some(container) = as_container(ty) {
            let name = cstr_to_rust((*(*container).container_type).name);
            if containers.contains(&name) {
                return;
            } else {
                containers.push(name);
            }
        }

        if let Some(pointer) = as_pointer(ty) {
            let name = cstr_to_rust((*(*pointer).pointer_type).name);
            if pointers.contains(&name) {
                return;
            } else {
                pointers.push(name);
            }
        }

        let mut obj = writer.object(rtti_name(ty).as_str());
        obj.value("kind", (*ty).kind.to_string().as_str());
        if let Some(compound) = as_compound(ty) {
            obj.value("version", (*compound).version);
            obj.value("flags", (*compound).flags);

            if !(*compound).message_handlers().is_empty() {
                let mut array = obj.array("messages");
                for message in (*compound).message_handlers() {
                    array.value(rtti_name((*message).message).as_str())
                }
                array.end();
            }

            if !(*compound).bases().is_empty() {
                let mut array = obj.array("bases");
                for base in (*compound).bases() {
                    let mut base_obj = array.object();
                    base_obj.value("name", rtti_name(base.base).as_str());
                    base_obj.value("offset", base.offset);
                    base_obj.end();
                }
                array.end();
            }

            if !(*compound).attributes().is_empty() {
                let mut array = obj.array("attrs");
                for attr in (*compound).attributes() {
                    let mut attr_obj = array.object();
                    if attr.base.is_null() {
                        attr_obj.value("category", cstr_to_rust(attr.name).as_str());
                        attr_obj.end();
                        continue;
                    }

                    attr_obj.value("name", cstr_to_rust(attr.name).as_str());
                    attr_obj.value("type", rtti_name(attr.base).as_str());
                    attr_obj.value("offset", attr.offset);
                    attr_obj.value("flags", attr.flags);

                    if !attr.min_value.is_null() {
                        attr_obj.value("min", cstr_to_rust(attr.min_value).as_str());
                    }
                    if !attr.max_value.is_null() {
                        attr_obj.value("max", cstr_to_rust(attr.max_value).as_str());
                    }
                    if !attr.getter.is_null() && !attr.setter.is_null() {
                        attr_obj.value("property", true);
                    }
                    attr_obj.end();
                }
                array.end();
            }
        } else if let Some(enum_) = as_enum(ty) {
            obj.value("size", (*enum_).size);
            let mut array = obj.array("values");
            for value in (*enum_).values() {
                let mut value_obj = array.object();
                value_obj.value("value", value.value);
                value_obj.value("name", cstr_to_rust(value.name).as_str());
                if let Some(aliases) = value.aliases() {
                    let mut array = value_obj.array("alias");
                    for alias in aliases {
                        array.value(alias.as_str());
                    }
                    array.end();
                }
                value_obj.end();
            }
            array.end();
        } else if let Some(atom) = as_primitive(ty) {
            obj.value("base_type", rtti_name((*atom).base_type).as_str());
        }
        obj.end();
    }
}

fn cstr_to_rust(cstr: *const c_char) -> String {
    if cstr.is_null() {
        String::new()
    } else {
        let cstr = unsafe { CStr::from_ptr(cstr) };
        cstr.to_str().unwrap().to_string()
    }
}
