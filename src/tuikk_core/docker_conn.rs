use arc_swap::ArcSwap;
use bollard::{self, API_DEFAULT_VERSION, ClientVersion, Docker};
use smart_default::SmartDefault;
use std::sync::{Arc, OnceLock};
use tuikk_macros::extend_base_err;
use serde::{Deserialize, Serialize};
#[extend_base_err]
pub enum DockerConnError {
    #[error("Docker URL is missing or empty")]
    MissingUrl,

    #[error("Docker daemon error: {0}")]
    Bollard(#[from] bollard::errors::Error),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum DefaultConnectionType {
    #[serde(rename = "http")]
    HTTP,

    #[default]
    #[serde(rename = "socket")]
    SOCKET,

    #[serde(rename = "local")]
    LOCAL,
}

#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerConfig {
    pub default_conn_type: DefaultConnectionType,
    pub url: Option<String>,
    #[default = 120]
    pub timeout: u64,
    #[default = 20]
    pub max_connections: i32,
}

#[derive(Debug, Clone)]
pub struct DockerConnection {
    pub url: Option<String>,
    pub timeout: u64,
    pub client_version: ClientVersion,
}

impl Default for DockerConnection {
    fn default() -> Self {
        Self {
            url: None,
            timeout: 120,
            client_version: API_DEFAULT_VERSION.clone(),
        }
    }
}
impl DockerConnection {
    pub fn connection(&self) {}

    fn local_conn() -> Result<Docker, DockerConnError> {
        Docker::connect_with_local_defaults().map_err(Into::into)
    }

    #[cfg(unix)]
    fn socket_conn() -> Result<Docker, DockerConnError> {
        Docker::connect_with_socket_defaults().map_err(Into::into)
    }

    fn http_conn(&self) -> Result<Docker, DockerConnError> {
        let url = self
            .url
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or(DockerConnError::MissingUrl)?;

        Docker::connect_with_http(url.trim(), self.timeout, &self.client_version)
            .map_err(Into::into)
    }
}
