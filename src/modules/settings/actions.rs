use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingsAction {
    Save,
    ReloadFromDisk,
    RestoreDefaults,
    PingDocker,
    PingCompleted(Result<String, String>),
}

