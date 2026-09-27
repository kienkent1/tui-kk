use arc_swap::ArcSwap;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tuikk_macros::extend_base_err;

use crate::tuikk_core::docker_conn::DockerConfig;
use crate::tuikk_core::key_map::KeyMap;
use crate::tuikk_core::ui_config::UiConfig;

#[extend_base_err]
pub enum ConfigError {
    #[error("Unable to determine the system configuration directory")]
    NoConfigDir,

    #[error("I/O error while handling config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML syntax error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
#[serde(default)]
pub struct AppConfig {
    #[default("default".to_owned())]
    pub theme: String,
    #[default("error".to_owned())]
    pub log_level: String,
    pub docker: DockerConfig,
    pub ui: UiConfig,
    #[serde(skip_serializing)]
    #[default(builtin_keymap())]
    pub keybindings: KeyMap,
}

const DEFAULT_KEYS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.config/keybindings.toml"
));

fn builtin_keymap() -> KeyMap {
    toml::from_str(DEFAULT_KEYS).expect(".config/keybindings.toml not valid")
}

fn merge_keymap(mut base: KeyMap, user: KeyMap) -> KeyMap {
    for (scope, binds) in user.0 {
        base.0.entry(scope).or_default().extend(binds);
    }
    base
}

// =========Constants=============
const CONFIG_FILE_NAME: &str = "config.toml";
static APP_CONFIG: OnceLock<ArcSwap<AppConfig>> = OnceLock::new();

impl AppConfig {
    fn finalize(mut self) -> Self {
        self.ui = self.ui.sanitized();
        self.keybindings = merge_keymap(builtin_keymap(), self.keybindings);
        self
    }

    pub fn init_global(config: AppConfig) {
        let _ = APP_CONFIG.set(ArcSwap::from_pointee(config));
    }

    pub fn global() -> Arc<AppConfig> {
        APP_CONFIG
            .get()
            .expect("AppConfig has not been initialized. Call init_global() first.")
            .load_full()
    }

    pub fn update_global(new_config: AppConfig) -> Result<(), ConfigError> {
        new_config.save_to_file()?;
        if let Some(swap) = APP_CONFIG.get() {
            swap.store(Arc::new(new_config));
        }
        Ok(())
    }
    /// Linux/macOS: ~/.config/tuikk/config.toml
    /// Windows: C:\Users\<User>\AppData\Roaming\tuikk\config.toml
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs =
            ProjectDirs::from("", "", env!("APP_PREFIX")).ok_or(ConfigError::NoConfigDir)?;

        let config_dir = proj_dirs.config_dir();

        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        Ok(config_dir.join(CONFIG_FILE_NAME))
    }

    pub fn load_from_file() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        if !path.exists() {
            let default_config = Self::default();
            default_config.save_to_file()?;
            return Ok(default_config.finalize());
        }

        let content = read_to_string(&path)?;

        let config: AppConfig = toml::from_str(&content)?;
        Ok(config.finalize())
    }

    pub fn load_or_default() -> Self {
        match Self::load_from_file() {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "Unable to read configuration file: {err}. Using default config."
                );
                Self::default().finalize()
            }
        }
    }

    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        write(path, content)?;
        Ok(())
    }
}
