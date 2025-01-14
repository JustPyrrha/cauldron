use chrono::prelude::Utc;
use codegen::{Block, Field, Module, Scope, Struct, Type, Variant};
use serde_json::{Map, Value};
use std::cmp::PartialEq;
use std::collections::HashMap;
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
enum RTTIToRustTypes {
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

impl From<&String> for RTTIToRustTypes {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "bool" => RTTIToRustTypes::bool,
            "int" => RTTIToRustTypes::int,
            "int8" => RTTIToRustTypes::int8,
            "int16" => RTTIToRustTypes::int16,
            "int32" => RTTIToRustTypes::int32,
            "int64" => RTTIToRustTypes::int64,
            "intptr" => RTTIToRustTypes::intptr,
            "uint" => RTTIToRustTypes::uint,
            "uint8" => RTTIToRustTypes::uint8,
            "uint16" => RTTIToRustTypes::uint16,
            "uint32" => RTTIToRustTypes::uint32,
            "uint64" => RTTIToRustTypes::uint64,
            "uint128" => RTTIToRustTypes::uint128,
            "uintptr" => RTTIToRustTypes::uintptr,
            "float" => RTTIToRustTypes::float,
            "double" => RTTIToRustTypes::double,
            "HalfFloat" => RTTIToRustTypes::HalfFloat,
            "tchar" => RTTIToRustTypes::tchar,
            "wchar" => RTTIToRustTypes::wchar,
            "ucs4" => RTTIToRustTypes::ucs4,
            "String" => RTTIToRustTypes::String,
            "WString" => RTTIToRustTypes::WString,
            &_ => RTTIToRustTypes::Unknown,
        }
    }
}

impl RTTIToRustTypes {
    pub fn to_string(&self) -> String {
        String::from(match self {
            RTTIToRustTypes::bool => "bool",
            RTTIToRustTypes::int => "i32",
            RTTIToRustTypes::int8 => "i8",
            RTTIToRustTypes::int16 => "i16",
            RTTIToRustTypes::int32 => "i32",
            RTTIToRustTypes::int64 => "i64",
            RTTIToRustTypes::intptr => "i64",
            RTTIToRustTypes::uint => "u32",
            RTTIToRustTypes::uint8 => "u8",
            RTTIToRustTypes::uint16 => "u16",
            RTTIToRustTypes::uint32 => "u32",
            RTTIToRustTypes::uint64 => "u64",
            RTTIToRustTypes::uint128 => "u128",
            RTTIToRustTypes::uintptr => "u64",
            RTTIToRustTypes::float => "f32",
            RTTIToRustTypes::double => "f64",
            RTTIToRustTypes::HalfFloat => "f32",
            RTTIToRustTypes::tchar => "char",
            RTTIToRustTypes::wchar => "char",
            RTTIToRustTypes::ucs4 => "u32",
            RTTIToRustTypes::String => "String",
            RTTIToRustTypes::WString => "String",
            _ => unreachable!(),
        })
    }
    pub fn to_type(&self) -> Type {
        Type::new(self.to_string().as_str())
    }
}

fn gen_bindings(game_id: &str) {
    let input: String = fs::read_to_string(format!("data/{}.json", game_id)).unwrap();
    let input: Value = serde_json::from_str(input.as_str()).unwrap();

    let mut scope = Scope::new();
    let mut module = scope.new_module(game_id);
    module.vis("pub");

    for (type_name, type_value) in input.as_object().unwrap() {
        let type_value = type_value.as_object().unwrap();
        match type_value["kind"].as_str().unwrap() {
            "primitive" => gen_primitive(&mut module, type_name, type_value),
            "enum flags" | "enum" => gen_enum(&mut module, type_name, type_value),
            "class" => gen_class(&mut module, type_name, type_value),
            "pointer" | "container" => gen_pointer(&mut module, type_name, type_value),
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
    if RTTIToRustTypes::from(name) != RTTIToRustTypes::Unknown {
        return;
    }

    let base_type = value["base_type"].as_str().unwrap();
    let base_type = RTTIToRustTypes::from(&base_type.to_string());
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

    let enum_ = scope
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
        enum_.push_variant(variant.clone());
    }

    let impl_scope = scope.new_impl(name);

    // workaround for https://gitlab.com/yovoslav/codegen/-/issues/15
    let value_fn = impl_scope
        .new_fn("value")
        .vis("pub")
        .arg_ref_self()
        .ret("u32");

    let mut block = Block::new("match self");

    for (var_ident, _, var_value) in &values {
        block.line(format!("{}::{} => {},", name, var_ident, var_value));
    }
    value_fn.push_block(block);

    let from_value_fn = impl_scope
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
    let mut category_structs = Vec::new();
    let class = scope
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
        for (idx, base) in value["bases"].as_array().unwrap().iter().enumerate() {
            let base = base.as_object().unwrap();
            let field_name = if idx == 0 {
                String::from("base")
            } else {
                format!("base{}", idx)
            };
            let mut field = Field::new(field_name.as_str(), base["name"].as_str().unwrap());
            field.doc(format!("offset: {}", base["offset"].as_i64().unwrap()));
            field.vis("pub");

            class.push_field(field.clone());
        }
    }

    if value.contains_key("attrs") {
        let attrs = collect_attr_categories(value["attrs"].as_array().unwrap());
        for (category, attrs) in attrs {
            if category.is_empty() {
                for attr in attrs {
                    let field = gen_attr_field(&attr);
                    class.push_field(field);
                }
            } else {
                let (category_type_name, category_struct) = gen_category(name, &category, &attrs);
                let mut field = Field::new(
                    format!("{}_category", category)
                        .as_str()
                        .to_lowercase()
                        .as_str(),
                    category_type_name.as_str(),
                );
                field.vis("pub");
                class.push_field(field.clone());
                category_structs.push(category_struct);
            }
        }
    }

    for category in category_structs {
        scope.push_struct(category);
    }

    // todo: msgs
}

fn collect_attr_categories(value: &Vec<Value>) -> HashMap<String, Vec<Map<String, Value>>> {
    let mut categories: HashMap<String, Vec<Map<String, Value>>> = HashMap::new();
    let mut last_category = String::new();
    for attr in value {
        let attr = attr.as_object().unwrap();
        if attr.contains_key("category") {
            last_category = attr["category"].as_str().unwrap().to_string();
            continue;
        }

        if !categories.contains_key(&last_category) {
            categories.insert(last_category.clone(), Vec::new());
        }

        categories
            .get_mut(&last_category)
            .unwrap()
            .push(attr.clone());
    }

    categories
}

fn gen_category(
    parent_name: &String,
    name: &String,
    value: &Vec<Map<String, Value>>,
) -> (String, Struct) {
    let name = format!("{}{}Category", parent_name, name);
    let mut category = Struct::new(name.as_str());
    category.derive("Debug");
    category.derive("Clone");
    category.vis("pub");
    category.doc(format!("attr category for {}", parent_name).as_str());

    for attr in value {
        let field = gen_attr_field(attr);
        category.push_field(field);
    }

    (name.clone(), category.clone())
}

fn gen_attr_field(attr: &Map<String, Value>) -> Field {
    let attr_type = if RTTIToRustTypes::from(&attr["type"].as_str().unwrap().to_string())
        != RTTIToRustTypes::Unknown
    {
        RTTIToRustTypes::from(&attr["type"].as_str().unwrap().to_string()).to_string()
    } else {
        attr["type"].as_str().unwrap().to_string()
    };
    let attr_type = replace_known_primitive_args(&attr_type);
    let attr_type = Type::new(&attr_type);

    let mut name = attr["name"].as_str().unwrap();
    // just some quick fixes for a few broken names
    if name == "3D" {
        name = "b3D";
    } else if name == "type" {
        name = "type_";
    }

    let mut field = Field::new(name, attr_type);
    field.doc(format!(
        "offset: {}, flags: {}",
        attr["offset"], attr["flags"]
    ));
    field.vis("pub");

    field.clone()
}

fn gen_pointer(scope: &mut Module, name: &String, value: &Map<String, Value>) {
    scope
        .new_struct(name)
        .generic("T")
        .vis("pub")
        .derive("Debug")
        .derive("Clone")
        .doc(value["kind"].as_str().unwrap())
        .field("marker", "std::marker::PhantomData<T>");
}

fn replace_known_primitive_args(raw: &String) -> String {
    if raw.contains("<") && raw.contains(">") {
        let split = raw.split("<").collect::<Vec<_>>();
        let generic = split[1].to_string().replace(">", "");
        if RTTIToRustTypes::from(&generic) != RTTIToRustTypes::Unknown {
            return format!(
                "{}<{}>",
                split[0],
                RTTIToRustTypes::from(&generic).to_string()
            );
        }
    }

    raw.clone()
}
