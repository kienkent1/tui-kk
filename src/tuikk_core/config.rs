use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Không thể xác định thư mục cấu hình hệ thống")]
    NoConfigDir,

    #[error("Lỗi I/O khi thao tác file config: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lỗi cú pháp file TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Lỗi serialize TOML: {0}")]
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
            docker_url: None, // Mặc định None -> dùng Local Socket
            timeout: 120,
            max_connections: 20,
            theme: "default".to_string(),
            default_conn_type: DefaultConnectionType::SOCKET,
        }
    }
}

impl AppConfig {
    /// 1. Xác định vị trí lưu trữ file config chuẩn theo hệ điều hành
    /// Linux/macOS: ~/.config/tuikk/config.toml
    /// Windows: C:\Users\<User>\AppData\Roaming\tuikk\config.toml
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = ProjectDirs::from("", "", "tuikk")
            .ok_or(ConfigError::NoConfigDir)?;
        
        let config_dir = proj_dirs.config_dir();
        
        // Tự động tạo thư mục nếu chưa tồn tại
        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        Ok(config_dir.join("config.toml"))
    }

    /// 2. Hàm đọc file config (Chi tiết từng bước, có kiểm tra lỗi)
    pub fn load_from_file() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;

        // Nếu file chưa tồn tại -> Ghi config mặc định và trả về
        if !path.exists() {
            let default_config = Self::default();
            default_config.save_to_file()?;
            return Ok(default_config);
        }

        // Đọc toàn bộ nội dung file thành String
        let content = read_to_string(&path)?;

        // Parse chuỗi TOML sang Struct AppConfig
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 3. Hàm đọc tiện lợi (Fallback an toàn cho TUI)
    /// Nếu có bất kỳ lỗi nào (file lỗi cú pháp, mất quyền đọc...), 
    /// app sẽ dùng Default chứ không bị crash!
    pub fn load_or_default() -> Self {
        match Self::load_from_file() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("[Warning] Không thể đọc file config: {err}. Dùng cấu hình mặc định.");
                Self::default()
            }
        }
    }

    /// 4. Ghi đè file config (Dùng khi user chỉnh setting trên TUI và bấm Save)
    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        write(path, content)?;
        Ok(())
    }
}
