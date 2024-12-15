use crate::{CauldronEnv, Plugin};
use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use std::env::current_dir;
use std::{fs, thread};
use std::fs::File;
use std::path::PathBuf;
use windows_sys::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
use focus::Focus;
use focus::hooks::dx12::Dx12Hooks;
use crate::core_plugin::{CauldronCore, CauldronUI};

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
                if path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(".dll")
                {
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

pub(crate) fn find_plugins(
    plugin_paths: &Vec<PathBuf>,
    _env: CauldronEnv,
) -> &Vec<Box<dyn Plugin + Send + Sync + 'static>> {
    static INTERNAL_PLUGINS: OnceCell<Vec<Box<dyn Plugin + Send + Sync + 'static>>> = OnceCell::new();
    &INTERNAL_PLUGINS.get_or_init(|| unsafe {
        let mut plugins: Vec<Box<dyn Plugin + Send + Sync + 'static>> = Vec::new();

        plugins.push(Box::new(CauldronCore {}));

        for path in plugin_paths {
            match libloading::Library::new(path) {
                Ok(lib) => {
                    let maybe_plugin_func = lib.get::<unsafe extern "C" fn() -> Box<
                        dyn Plugin + Send + Sync + 'static,
                    >>(b"__cauldron_api__plugin");

                    match maybe_plugin_func {
                        Ok(plugin_func) => {
                            // let plugin_main_func = lib
                            //     .get::<unsafe extern "C" fn(CauldronEnv, PluginMainReason) -> ()>(
                            //         b"__cauldron_api__main",
                            //     )
                            //     .expect("malformed plugin missing main export.");
                            // plugin_main_func(env.clone(), PluginMainReason::Load);
                            plugins.push(plugin_func());
                        }
                        Err(_) => {
                            warn!("cauldron: {:?} is not a cauldron plugin, unloading.", path);
                            lib.close().unwrap();
                        }
                    }
                }
                Err(e) => {
                    error!("libloading error: {:?}", e);
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
    ])
    .unwrap();

    info!("Starting Cauldron v{}...", env!("CARGO_PKG_VERSION"));
    // load plugins
    let game_dir = current_dir().unwrap();
    debug!("{:?}", game_dir);
    let plugin_paths = plugin_paths(&game_dir);
    info!("Loading plugins...");
    let plugins = find_plugins(&plugin_paths, CauldronEnv::new());

    // todo: fix this crashing lmao
    // thread::spawn(|| {
    //     focus::util::enable_debug_interface(false);
    //     Focus::builder().with::<Dx12Hooks>(CauldronUI::new()).build().apply().unwrap();
    //     //todo: split core plugin and ui
    // });

    // print plugins table
    {
        let mut builder = tabled::builder::Builder::new();
        builder.push_record(["Order", "Id", "Version", "Name", "Authors", "Description"]);

        plugins.iter().enumerate().for_each(|(i, plugin)| {
            let meta = plugin.meta();
            let name = &meta.name.unwrap_or(String::new());
            let authors = &meta.authors.unwrap_or(Vec::new()).join(", ");
            let description = &meta.description.unwrap_or(String::new());

            builder.push_record([
                format!("{}", i),
                format!("{}", &meta.id),
                format!("{}", &meta.version),
                format!("{}", name),
                format!("{}", authors),
                format!("{}", description),
            ]);
        });
        info!("Found {} plugins:\n{}", plugins.len(), builder.build());
    }

    // do early init
    plugins.iter().for_each(|plugin| {
        plugin.early_init();
    });
}
