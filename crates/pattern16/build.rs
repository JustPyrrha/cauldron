use std::fs::OpenOptions;
use std::io::Write;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=vendor/Pattern16/include/");

    cc::Build::new()
        .cpp(true)
        .file("pat16/pat16.cpp")
        .include("vendor/Pattern16/include")
        .include("pat16")
        .std("c++17")
        .compile("pat16.a");

    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );

    let _ = bindgen::Builder::default()
        .header("wrapper.hpp")
        .allowlist_function("Pat16::.*")
        .allowlist_function("Pat16::Impl::.*")
        .opaque_type("Pattern16::Impl::CacheSerialized")
        .opaque_type("Pattern16::Impl::Frequencies16")
        .opaque_type("std::.*")
        .clang_arg("-Ivendor/Pattern16/include")
        .clang_arg("-Ipat16")
        .clang_arg("-IC:/Program Files/LLVM/lib/clang/19/include") // todo: fix this, its is gross and i hate it
        .clang_arg("-std=c++17")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("gen failed")
        .write_to_file("src/bindings.rs");

    // todo: replace std_string with
    let content = fs::read_to_string("src/bindings.rs").unwrap();
    let content = content
        .replace(
            "pub type std_string = __BindgenOpaqueArray<u64, 4usize>;",
            "",
        )
        .replace("std_string", "*const ::std::os::raw::c_char");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("src/bindings.rs")
        .unwrap();
    file.write(content.as_bytes()).unwrap();
}
