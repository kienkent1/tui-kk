
use bollard::{self, ClientVersion, Docker, errors::Error};
#[derive(Debug, Clone)]
struct DockerConnection {
    url: Option<String>,
    timeout: u64,
    client_version: ClientVersion
}

impl Default for DockerConnection {
    fn default() -> Self {
        Self {
            url: None,      
            timeout: 60,  
            client_version: ClientVersion::API_DEFAULT_VERSION
        }
    }
}
impl DockerConnection{
    pub fn connection() {}

    fn local_conn() -> Result<Docker, Error>{
        Docker::connect_with_local_defaults()
    }

    #[cfg(unix)]
    fn socket_conn() -> Result<Docker, Error>{
        Docker::connect_with_socket_defaults()
    }

    fn http_conn(self) -> Result<Docker, Error> {
        if self.url.is_none(){
            return Err(Error::(
                "Docker URL is missing/None".to_string()
            ));
        }
        Docker::connect_with_http(self.url, self.timeout, &self.client_version )
    }
}


