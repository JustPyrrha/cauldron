use std::env::current_dir;
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use log::{debug, info, warn};
use once_cell::sync::OnceCell;
use windows_sys::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
use crate::Plugin;

pub(crate) fn plugin_paths(game_dir: &PathBuf) -> &Vec<PathBuf> {
    static PATHS: OnceCell<Vec<PathBuf>> = OnceCell::new();
    PATHS.get_or_init(|| {
        let plugins_dir = game_dir.join("plugins");
        if plugins_dir.exists() && plugins_dir.is_dir() {
            debug!("plugins dir exists, loading plugins. {:?}", plugins_dir);
            let paths = fs::read_dir(plugins_dir).unwrap();
            let mut out = Vec::new();
            for path in paths {
                let path = path.unwrap().path();
                if path.file_name().unwrap().to_str().unwrap().ends_with(".dll") {
                    out.push(path);
                }
            }

            out
        } else {
            warn!("cauldron: plugins directory does not exist.");
            Vec::new()
        }
    })
}

pub(crate) fn plugins(plugin_paths: &Vec<PathBuf>) -> &Vec<Box<dyn Plugin + Send + Sync + 'static>> {
    static PLUGINS: OnceCell<Vec<Box<dyn Plugin + Send + Sync + 'static>>> = OnceCell::new();
    &PLUGINS.get_or_init(|| unsafe {
        let mut plugins: Vec<Box<dyn Plugin + Send + Sync + 'static>> = Vec::new();
        for path in plugin_paths {
            let lib = libloading::Library::new(path).unwrap();
            let maybe_plugin_func = lib.get::<unsafe extern "C" fn() -> Box<dyn Plugin + Send + Sync + 'static>>(b"__cauldron_api__plugin");
            match maybe_plugin_func {
                Ok(plugin_func) => {
                    plugins.push(plugin_func());
                }
                Err(_) => {
                    warn!("cauldron: {:?} is not a cauldron plugin, unloading.", path);
                    lib.close().unwrap();
                }
            }
        }

        // todo: sort by dependency requirements/actually setup load order lol
        plugins
    })
}

pub(crate) fn on_dll_attach() {
    // setup logging
    unsafe {
        AllocConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    // todo: move level config to an actual config file
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            log::LevelFilter::Trace,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(
            log::LevelFilter::Trace,
            simplelog::Config::default(),
            File::create("cauldron.log").unwrap(),
        ),
    ]).unwrap();

    info!("Starting Cauldron v{}...", env!("CARGO_PKG_VERSION"));
    // load plugins
    let game_dir = current_dir().unwrap();
    debug!("{:?}", game_dir);
    let plugin_paths = plugin_paths(&game_dir);
    let plugins = plugins(&plugin_paths);

    info!("Loading {} plugins...", plugins.len());

    // do early init
    plugins.iter().for_each(|plugin| {
        plugin.early_init();
    });
}