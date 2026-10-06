use arc_swap::ArcSwap;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tuikk_macros::extend_base_err;

use crate::shared::helpers::ext_log::ResultExt;
use crate::tuikk_core::docker_conn::DockerConfig;
use crate::tuikk_core::key_map::KeyMap;
use crate::tuikk_core::themes::ThemeColor;
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

    #[error("Theme file parse error: {0}")]
    ThemeParse(String),

    #[error("Json serialization error: {0}")]
    Json5(#[from] json5::Error),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryMode {
    #[default]
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
#[serde(default)]
pub struct AppConfig {
    // Theme & style
    #[default("dark".to_owned())]
    pub theme: String,
    #[serde(skip_serializing)]
    #[default(ThemeColor::default(None))]
    pub theme_color: ThemeColor,
    pub ui: UiConfig,

    // Logging
    #[default("error".to_owned())]
    pub log_level: String,
    pub log_dir: Option<String>,

    pub docker: DockerConfig,

    #[serde(skip_serializing)]
    #[default(KeyMap::default())]
    pub keybindings: KeyMap,
    pub query_mode: QueryMode,
}

// Key binding
const KEY_FILE_NAME: &str = "keybindings.toml";

fn builtin_keymap() -> Result<KeyMap, ConfigError> {
    let path = AppConfig::get_path(KEY_FILE_NAME)?;

    if !path.exists() {
        return Ok(KeyMap::default());
    }

    let content = read_to_string(&path)?;
    let keymap = toml::from_str(&content).map_err(|e| {
        tracing::warn!("Invalid keybindings file at {}: {e}", path.display());
        e
    })?;

    Ok(keymap)
}

fn merge_keymap(mut base: KeyMap, user: KeyMap) -> KeyMap {
    for (scope, binds) in user.0 {
        base.0.entry(scope).or_default().extend(binds);
    }
    base
}

// Theme color
const THEME_JSON5_FILE_NAME: &str = "theme.json5";
const THEME_JSON_FILE_NAME: &str = "theme.json";
fn buildin_theme_color(theme: String) -> Result<ThemeColor, ConfigError> {
    let path = [THEME_JSON5_FILE_NAME, THEME_JSON_FILE_NAME]
        .iter()
        .map(|name| AppConfig::get_path(name))
        .filter_map(|r| r.ok())
        .find(|p| p.exists());

    let Some(path) = path else {
        return Ok(ThemeColor::default(None));
    };

    let content = read_to_string(&path)?;
    let themes: HashMap<String, ThemeColor> = json5::from_str(&content).map_err(|e| {
        tracing::warn!("Invalid theme file at {}: {e}", path.display());
        ConfigError::ThemeParse(e.to_string())
    })?;
    let theme_color = themes.get(&theme).copied().unwrap_or_default();
    Ok(theme_color)
}

// =========Constants=============
const CONFIG_FILE_NAME: &str = "config.toml";
static APP_CONFIG: OnceLock<ArcSwap<AppConfig>> = OnceLock::new();

impl AppConfig {
    fn finalize(mut self) -> Self {
        self.ui = self.ui.sanitized();
        let base = builtin_keymap().unwrap_or_default();
        self.keybindings = merge_keymap(base, self.keybindings);

        self.theme_color = buildin_theme_color(self.theme.clone()).log_warn().unwrap();
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
        AppConfig::get_path(CONFIG_FILE_NAME)
    }

    pub fn get_path(file_name: &str) -> Result<PathBuf, ConfigError> {
        let local_dir = PathBuf::from(".config");
        let local_file = local_dir.join(file_name);

        if local_file.exists() {
            return Ok(local_file);
        }

        if local_dir.is_dir() {
            return Ok(local_file);
        }

        let proj_dirs =
            ProjectDirs::from("", "", env!("APP_PREFIX")).ok_or(ConfigError::NoConfigDir)?;

        let config_dir = proj_dirs.config_dir();

        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        Ok(config_dir.join(file_name))
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
        self.save_file(path, content)
    }

    pub fn save_file(&self, path: PathBuf, content: String) -> Result<(), ConfigError> {
        write(path, content).map_err(|e| ConfigError::Io(e))
    }
}
