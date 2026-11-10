use std::sync::{Arc, OnceLock};

use bollard::Docker;

use crate::{
    modules::{
        containers::container_service::ContainerService,
        dashboard::dashboard_service::DashboardService, images::image_service::ImageService,
    },
    tuikk_core::docker_conn::{DockerConnError, DockerConnection},
};

static INSTANCE: OnceLock<AppServices> = OnceLock::new();

pub struct AppServices {
    pub docker: Arc<Docker>,
    pub dashboard: Arc<DashboardService>,
    pub containers: Arc<ContainerService>,
    pub images: Arc<ImageService>,
}

impl AppServices {
    pub fn init() -> Result<(), DockerConnError> {
        let docker = DockerConnection::global()?;
        let dashboard = Arc::new(DashboardService::new(docker.clone()));
        let containers = Arc::new(ContainerService::new(docker.clone()));
        let images = Arc::new(ImageService::new(docker.clone()));

        INSTANCE
            .set(AppServices {
                docker,
                dashboard,
                containers,
                images,
            })
            .map_err(|_| DockerConnError::AlreadyInitialized)
    }

    pub fn get() -> &'static AppServices {
        INSTANCE.get().expect("AppServices::init() was not called")
    }

    pub fn try_get() -> Option<&'static AppServices> {
        INSTANCE.get()
    }
}
