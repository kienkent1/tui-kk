use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use tuikk_macros::extend_base_err;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub theme: String,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            log_level: "error".to_owned(),
        }
    }
}

// =========Constants=============
const CONFIG_FILE_NAME: &str = "config.toml";

impl AppConfig {
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
            return Ok(default_config);
        }

        let content = read_to_string(&path)?;

        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        match Self::load_from_file() {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "Unable to read configuration file: {err}. Using default config."
                );
                Self::default()
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
