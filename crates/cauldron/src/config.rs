use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::File;
use std::io::Write;

use crate::version::CauldronGameType;

/// Used to determine the config version before fully deserializing.
#[derive(Debug, Deserialize)]
pub struct CauldronConfigVersionOnly {
    pub config_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CauldronConfig {
    /// DO NOT EDIT - Used to identify which version of the config is used.
    #[serde(default)]
    pub config_version: u32,

    #[serde(default)]
    pub logging: CauldronConfigLoggingSection,
    #[serde(default)]
    pub ui: CauldronConfigUiSeciton,
    #[serde(default)]
    pub game: Option<CauldronConfigGameSection>,
}

impl Default for CauldronConfig {
    fn default() -> CauldronConfig {
        CauldronConfig {
            config_version: 0,
            logging: CauldronConfigLoggingSection::default(),
            ui: CauldronConfigUiSeciton::default(),
            game: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(usize)]
pub enum LogLevelConfig {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevelConfig {
    pub fn from_log(level: log::LevelFilter) -> LogLevelConfig {
        match level {
            log::LevelFilter::Off => LogLevelConfig::Off,
            log::LevelFilter::Error => LogLevelConfig::Error,
            log::LevelFilter::Warn => LogLevelConfig::Warn,
            log::LevelFilter::Info => LogLevelConfig::Info,
            log::LevelFilter::Debug => LogLevelConfig::Debug,
            log::LevelFilter::Trace => LogLevelConfig::Trace,
        }
    }

    pub fn to_log(&self) -> log::LevelFilter {
        match self {
            LogLevelConfig::Off => log::LevelFilter::Off,
            LogLevelConfig::Error => log::LevelFilter::Error,
            LogLevelConfig::Warn => log::LevelFilter::Warn,
            LogLevelConfig::Info => log::LevelFilter::Info,
            LogLevelConfig::Debug => log::LevelFilter::Debug,
            LogLevelConfig::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CauldronConfigLoggingSection {
    pub show_console: bool,
    pub console_level: LogLevelConfig,

    pub file_level: LogLevelConfig,
    pub file_path: String,
}

impl Default for CauldronConfigLoggingSection {
    fn default() -> CauldronConfigLoggingSection {
        CauldronConfigLoggingSection {
            show_console: true,
            console_level: LogLevelConfig::Info,
            file_level: LogLevelConfig::Info,
            file_path: "cauldron/cauldron.log".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CauldronConfigGameSection {
    /// Override the detected game type.
    pub override_game: Option<CauldronGameType>,
    /// Override the detected game version.
    pub override_version: Option<String>,
}

impl Default for CauldronConfigGameSection {
    fn default() -> CauldronConfigGameSection {
        CauldronConfigGameSection {
            override_game: None,
            override_version: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CauldronConfigUiSeciton {
    pub enabled: bool,
    pub key: String,
    pub enable_dx12_debug: bool,
    pub enable_dx12_debug_gpu_validation: bool,
}

impl Default for CauldronConfigUiSeciton {
    fn default() -> Self {
        CauldronConfigUiSeciton {
            enabled: true,
            key: "`".to_string(),
            enable_dx12_debug: true,
            enable_dx12_debug_gpu_validation: false,
        }
    }
}

pub(crate) fn load_config() -> CauldronConfig {
    let dir = current_dir().unwrap();
    let dir = dir.join("cauldron");
    let file = dir.join("cauldron.toml");

    let config = if file.exists() {
        toml::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
    } else {
        let out =
            toml_edit::ser::to_string_pretty::<CauldronConfig>(&CauldronConfig::default()).unwrap();
        let mut file = File::create(file).unwrap();
        write!(file, "{}", out).unwrap();
        CauldronConfig::default()
    };

    config
}
