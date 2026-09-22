use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultConnectionType {
    #[serde(rename = "http")]
    HTTP,

    #[serde(rename = "socket")]
    SOCKET,

    #[serde(rename = "local")]
    LOCAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_conn_type: DefaultConnectionType,
    pub docker_url: Option<String>,
    pub timeout: u64,
    pub max_connections: usize,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            docker_url: None,
            timeout: 120,
            max_connections: 20,
            theme: "default".to_string(),
            default_conn_type: DefaultConnectionType::SOCKET,
        }
    }
}

impl AppConfig {
    /// Linux/macOS: ~/.config/tuikk/config.toml
    /// Windows: C:\Users\<User>\AppData\Roaming\tuikk\config.toml
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = ProjectDirs::from("", "", "tuikk").ok_or(ConfigError::NoConfigDir)?;

        let config_dir = proj_dirs.config_dir();

        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        Ok(config_dir.join("config.toml"))
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
                eprintln!(
                    "[Warning] Unable to read configuration file: {err}. Using default config."
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
