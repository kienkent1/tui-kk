use arc_swap::ArcSwap;
use bollard::{self, ClientVersion, Docker, API_DEFAULT_VERSION};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::sync::{Arc, OnceLock};
use tracing::instrument;
use tuikk_macros::extend_base_err;

use crate::shared::base::ext_log::ResultExt;

#[extend_base_err]
pub enum DockerConnError {
    #[error("Docker address or path is missing, empty or invalid")]
    MissingAddrOrPath,

    #[error("Docker connection has not been initialized")]
    NotInitialized,

    #[error("Docker connection has already been initialized")]
    AlreadyInitialized,

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
    pub addr_or_path: Option<String>,
    #[default = 120]
    pub timeout: u64,
    pub client_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DockerConnection {
    pub config: DockerConfig,
}

static DOCKER_SWAP: OnceLock<ArcSwap<Docker>> = OnceLock::new();

impl DockerConnection {
    pub fn init_global(config: DockerConfig) -> Result<(), DockerConnError> {
        let docker = Self { config }.connection()?;

        DOCKER_SWAP
            .set(ArcSwap::from_pointee(docker))
            .map_err(|_| DockerConnError::AlreadyInitialized)
            .log_err()?;

        std::result::Result::Ok(())
    }

    pub fn global() -> Result<Arc<Docker>, DockerConnError> {
        DOCKER_SWAP
            .get()
            .map(ArcSwap::load_full)
            .ok_or(DockerConnError::NotInitialized)
            .log_err()
    }

    pub fn reload_global(config: DockerConfig) -> Result<(), DockerConnError> {
        let docker = Self { config }.connection()?;

        let docker_swap = DOCKER_SWAP
            .get()
            .ok_or(DockerConnError::NotInitialized)
            .log_err()?;

        docker_swap.store(Arc::new(docker));

        std::result::Result::Ok(())
    }

    pub fn is_initialized() -> bool {
        DOCKER_SWAP.get().is_some()
    }

    /// Health-check thực sự kết nối tới Daemon
    pub async fn ping() -> Result<(), DockerConnError> {
        let docker = Self::global()?;
        docker.ping().await.map_err(DockerConnError::Bollard)?;
        std::result::Result::Ok(())
    }

    #[instrument(skip(self), err)]
    pub fn connection(&self) -> Result<Docker, DockerConnError> {
        match self.config.default_conn_type {
            DefaultConnectionType::LOCAL => self.local_conn(),
            DefaultConnectionType::HTTP => self.http_conn(),
            _ => self.socket_conn(),
        }
    }

    fn addr_or_path(&self) -> Option<&str> {
        self.config
            .addr_or_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    fn parse_client_version(&self) -> ClientVersion {
        self.config
            .client_version
            .as_deref()
            .and_then(|c| {
                let parts: Vec<&str> = c.split('.').collect();
                if parts.len() == 2 {
                    let major = parts[0].parse().ok()?;
                    let minor = parts[1].parse().ok()?;

                    Some(ClientVersion {
                        major_version: major,
                        minor_version: minor,
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(|| API_DEFAULT_VERSION.clone())
    }

    fn local_conn(&self) -> Result<Docker, DockerConnError> {
        match self.addr_or_path() {
            Some(v) => {
                Docker::connect_with_local(v, self.config.timeout, &self.parse_client_version())
                    .map_err(DockerConnError::Bollard)
            }
            None => Docker::connect_with_local_defaults().map_err(DockerConnError::Bollard),
        }
    }

    #[cfg(unix)]
    fn socket_conn(&self) -> Result<Docker, DockerConnError> {
        match self.addr_or_path() {
            Some(v) => {
                Docker::connect_with_socket(v, self.config.timeout, &self.parse_client_version())
                    .map_err(DockerConnError::Bollard)
            }
            None => Docker::connect_with_socket_defaults().map_err(DockerConnError::Bollard),
        }
    }

    #[cfg(not(unix))]
    fn socket_conn(&self) -> Result<Docker, DockerConnError> {
        self.local_conn()
    }

    fn http_conn(&self) -> Result<Docker, DockerConnError> {
        let raw_addr = self
            .addr_or_path()
            .ok_or(DockerConnError::MissingAddrOrPath)?;

        let normalized_url = if let Some(stripped) = raw_addr.strip_prefix("tcp://") {
            format!("http://{stripped}")
        } else if raw_addr.starts_with("http://") || raw_addr.starts_with("https://") {
            raw_addr.to_string()
        } else {
            return Err(DockerConnError::MissingAddrOrPath);
        };

        Docker::connect_with_http(
            &normalized_url,
            self.config.timeout,
            &self.parse_client_version(),
        )
        .map_err(DockerConnError::Bollard)
    }
}
