use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::Command;

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
        _ => print_help()
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
        .args(&["build"])
        .status()?;
    if !status.success() {
        Err("cargo build failed")?;
    }

    let hfw_dir = Path::new("D:/SteamLibrary/steamapps/common/Horizon Forbidden West Complete Edition");
    let plugins_dir = hfw_dir.join("plugins");

    let core_outputs = vec![
        project_root().join("target/x86_64-pc-windows-msvc/debug/cauldron.dll"),
        project_root().join("target/x86_64-pc-windows-msvc/debug/cauldron.pdb"),
        project_root().join("target/x86_64-pc-windows-msvc/debug/version.dll"),
        project_root().join("target/x86_64-pc-windows-msvc/debug/version.pdb"),
    ];
    let plugin_outputs = vec![
        project_root().join("target/x86_64-pc-windows-msvc/debug/pulse.dll"),
        project_root().join("target/x86_64-pc-windows-msvc/debug/pulse.pdb"),
    ];

    core_outputs.iter().for_each(|path| {
        fs::copy(path, hfw_dir.join(path.file_name().unwrap())).expect("Could not copy cauldron");
    });

    if !&plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)?;
    }
    plugin_outputs.iter().for_each(|path| {
        fs::copy(path, &plugins_dir.join(path.file_name().unwrap())).expect("Could not copy plugins");
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