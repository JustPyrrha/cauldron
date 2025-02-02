use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

type DynError = Box<dyn std::error::Error>;

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("hfw") => hfw_task()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!("
Tasks:
\thfw - build cauldron and copy it to the hfw directory along with dev plugins. (you may need to change the dir this copies to in `xtask/src/main.rs`)
    ");
}

fn hfw_task() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(&["build", "--features", "hfw"])
        .status()?;
    if !status.success() {
        Err("cargo build failed")?;
    }

    let hfw_dir =
        Path::new("D:/SteamLibrary/steamapps/common/Horizon Forbidden West Complete Edition");
    let cauldron_dir = hfw_dir.join("cauldron");
    let plugins_dir = cauldron_dir.join("plugins");
    let pulse_dir = plugins_dir.join("pulse");
    let debug_out = project_root().join("target/x86_64-pc-windows-msvc/debug/");

    let root_outputs = vec![debug_out.join("version.dll"), debug_out.join("version.pdb")];
    let core_outputs = vec![
        debug_out.join("cauldron.dll"),
        debug_out.join("cauldron.pdb"),
    ];
    let pulse_outputs = vec![debug_out.join("pulse.dll"), debug_out.join("pulse.pdb")];
    let plugin_outputs = vec![
        debug_out.join("legacy.dll"),
        debug_out.join("legacy.pdb"),
        debug_out.join("hello_cauldron.dll"),
        debug_out.join("hello_cauldron.pdb"),
    ];

    if !&pulse_dir.exists() {
        fs::create_dir_all(&pulse_dir)?;
    }

    root_outputs.iter().for_each(|path| {
        fs::copy(path, hfw_dir.join(path.file_name().unwrap())).unwrap();
    });
    core_outputs.iter().for_each(|path| {
        fs::copy(path, cauldron_dir.join(path.file_name().unwrap())).unwrap();
    });
    pulse_outputs.iter().for_each(|path| {
        fs::copy(path, pulse_dir.join(path.file_name().unwrap())).unwrap();
    });
    plugin_outputs.iter().for_each(|path| {
        fs::copy(path, plugins_dir.join(path.file_name().unwrap())).unwrap();
    });

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
