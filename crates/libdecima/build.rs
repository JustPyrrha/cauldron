use chrono::prelude::Utc;
use codegen::{Block, Module, Scope, Type, Variant};
use serde_json::{Map, Value};
use std::cmp::PartialEq;
use std::fs::File;
use std::io::Write;
use std::{env, fs};

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/*.json");

    gen_bindings("hfw");
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
enum BuiltInTypes {
    bool,
    int,
    int8,
    int16,
    int32,
    int64,
    intptr,
    uint,
    uint8,
    uint16,
    uint32,
    uint64,
    uint128,
    uintptr,
    float,
    double,
    HalfFloat,
    tchar,
    wchar,
    ucs4,
    String,
    WString,

    Unknown,
}

impl From<&String> for BuiltInTypes {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "bool" => BuiltInTypes::bool,
            "int" => BuiltInTypes::int,
            "int8" => BuiltInTypes::int8,
            "int16" => BuiltInTypes::int16,
            "int32" => BuiltInTypes::int32,
            "int64" => BuiltInTypes::int64,
            "intptr" => BuiltInTypes::intptr,
            "uint" => BuiltInTypes::uint,
            "uint8" => BuiltInTypes::uint8,
            "uint16" => BuiltInTypes::uint16,
            "uint32" => BuiltInTypes::uint32,
            "uint64" => BuiltInTypes::uint64,
            "uint128" => BuiltInTypes::uint128,
            "uintptr" => BuiltInTypes::uintptr,
            "float" => BuiltInTypes::float,
            "double" => BuiltInTypes::double,
            "HalfFloat" => BuiltInTypes::HalfFloat,
            "tchar" => BuiltInTypes::tchar,
            "wchar" => BuiltInTypes::wchar,
            "ucs4" => BuiltInTypes::ucs4,
            "String" => BuiltInTypes::String,
            "WString" => BuiltInTypes::WString,
            &_ => BuiltInTypes::Unknown,
        }
    }
}

impl BuiltInTypes {
    pub fn to_type(&self) -> Type {
        Type::new(match self {
            BuiltInTypes::bool => "bool",
            BuiltInTypes::int => "i32",
            BuiltInTypes::int8 => "i8",
            BuiltInTypes::int16 => "i16",
            BuiltInTypes::int32 => "i32",
            BuiltInTypes::int64 => "i64",
            BuiltInTypes::intptr => "i64",
            BuiltInTypes::uint => "u32",
            BuiltInTypes::uint8 => "u8",
            BuiltInTypes::uint16 => "u16",
            BuiltInTypes::uint32 => "u32",
            BuiltInTypes::uint64 => "u64",
            BuiltInTypes::uint128 => "u128",
            BuiltInTypes::uintptr => "u64",
            BuiltInTypes::float => "f32",
            BuiltInTypes::double => "f64",
            BuiltInTypes::HalfFloat => "f32",
            BuiltInTypes::tchar => "char",
            BuiltInTypes::wchar => "char",
            BuiltInTypes::ucs4 => "u32",
            BuiltInTypes::String => "String",
            BuiltInTypes::WString => "String",
            _ => unreachable!(),
        })
    }
}

fn gen_bindings(game_id: &str) {
    let input: String = fs::read_to_string(format!("data/{}.json", game_id)).unwrap();
    let input: Value = serde_json::from_str(input.as_str()).unwrap();

    let mut scope = Scope::new();
    let mut module = scope.new_module(game_id);

    for (type_name, type_value) in input.as_object().unwrap() {
        let type_value = type_value.as_object().unwrap();
        match type_value["kind"].as_str().unwrap() {
            "primitive" => gen_primitive(&mut module, type_name, type_value),
            "enum flags" | "enum" => gen_enum(&mut module, type_name, type_value),
            "class" => gen_class(&mut module, type_name, type_value),
            _ => {}
        };
    }

    let mut file =
        File::create(format!("{}/{}.rs", env::var("OUT_DIR").unwrap(), game_id)).unwrap();
    writeln!(file, "/// GENERATED FILE. DO NOT EDIT.").unwrap();
    writeln!(file, "/// ").unwrap();
    writeln!(
        file,
        "/// Generated on {} by {}",
        Utc::now().format("%+"),
        "https://github.com/JustPyrrha/cauldron"
    )
    .unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "").unwrap();
    write!(file, "{}", scope.to_string()).unwrap();
    writeln!(file, "").unwrap();
}

fn gen_primitive(scope: &mut Module, name: &String, value: &Map<String, Value>) {
    if BuiltInTypes::from(name) != BuiltInTypes::Unknown {
        return;
    }

    let base_type = value["base_type"].as_str().unwrap();
    let base_type = BuiltInTypes::from(&base_type.to_string());
    let base_type = base_type.to_type();

    scope
        .new_struct(name)
        .repr("C")
        .derive("Debug")
        .derive("Clone")
        .vis("pub")
        .tuple_field(base_type);
}

fn gen_enum(scope: &mut Module, name: &String, value: &Map<String, Value>) {
    let values = value["values"].as_array().unwrap();

    let values = values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let value = value.as_object().unwrap();
            let var_name =
                if let Ok(_) = syn::parse_str::<syn::Ident>(value["name"].as_str().unwrap()) {
                    value["name"].as_str().unwrap().to_string()
                } else if let Ok(_) = syn::parse_str::<syn::Ident>(
                    format!("_{}", value["name"].as_str().unwrap()).as_str(),
                ) {
                    format!("_{}", value["name"].as_str().unwrap())
                } else if let Ok(_) = syn::parse_str::<syn::Ident>(
                    value["name"]
                        .as_str()
                        .unwrap()
                        .to_string()
                        .replace(" ", "_")
                        .replace("-", "_")
                        .as_str(),
                ) {
                    value["name"]
                        .as_str()
                        .unwrap()
                        .to_string()
                        .replace(" ", "_")
                        .replace("-", "_")
                        .to_string()
                } else if let Ok(_) = syn::parse_str::<syn::Ident>(
                    format!(
                        "_{}",
                        value["name"]
                            .as_str()
                            .unwrap()
                            .to_string()
                            .replace(" ", "_")
                            .replace("-", "_")
                            .as_str()
                    )
                    .as_str(),
                ) {
                    format!(
                        "_{}",
                        value["name"]
                            .as_str()
                            .unwrap()
                            .to_string()
                            .replace(" ", "_")
                            .replace("-", "_")
                            .as_str()
                    )
                    .to_string()
                } else {
                    format!("_{}", idx)
                };

            (
                var_name,
                value["name"].as_str().unwrap().to_string(),
                value["value"].as_i64().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let mut enum_ = scope
        .new_enum(name)
        .derive("Debug")
        .derive("PartialEq")
        .derive("Eq")
        .derive("Clone")
        .vis("pub");

    for (var_ident, var_name, var_value) in &values {
        let mut variant = Variant::new(var_ident);
        variant.annotation(format!("/// Value: {}", var_value));
        variant.annotation(format!("/// Name: {}", var_name));
        enum_.push_variant(variant);
    }

    let mut impl_scope = scope.new_impl(name);

    // workaround for https://gitlab.com/yovoslav/codegen/-/issues/15
    let mut value_fn = impl_scope
        .new_fn("value")
        .vis("pub")
        .arg_ref_self()
        .ret("u32");

    let mut block = Block::new("match self");

    for (var_ident, _, var_value) in &values {
        block.line(format!("{}::{} => {},", name, var_ident, var_value));
    }
    value_fn.push_block(block);

    let mut from_value_fn = impl_scope
        .new_fn("from_value")
        .vis("pub")
        .arg("value", "u32")
        .ret("Option<Self>");

    let mut block = Block::new("match value");

    for (var_ident, _, var_value) in &values {
        block.line(format!("{} => Some({}::{}),", var_value, name, var_ident));
    }

    block.line("_ => None,");

    from_value_fn.push_block(block);
}

fn gen_class(scope: &mut Module, name: &String, value: &Map<String, Value>) {
    let mut class = scope
        .new_struct(name)
        .doc(
            format!(
                "version: {}, flags: {}",
                value["version"].as_i64().unwrap(),
                value["flags"].as_i64().unwrap()
            )
            .as_str(),
        )
        .derive("Debug")
        .derive("Clone")
        .vis("pub");

    if value.contains_key("bases") {
        for base in value["bases"].as_array().unwrap() {
            let base = base.as_object().unwrap();
            let field_name = if base["offset"].as_i64().unwrap() == 0 {
                String::from("base")
            } else {
                format!("base_{}", base["offset"].as_i64().unwrap())
            };

            class.field(field_name.as_str(), base["name"].as_str().unwrap());
        }
    }

    // todo: msgs, attrs
}
