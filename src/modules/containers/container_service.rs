use std::sync::Arc;

use bollard::Docker;

use crate::{modules::containers::container_dto::ContainerDto, shared::base::base_err::BaseErr};

pub struct ContainerService {
    conn: Arc<Docker>,
}

impl ContainerService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }
    pub fn async get_containers(&self) -> Result<Vec<ContainerDto>, BaseErr>;
    pub fn async get_container(&self, id: &str) -> Result<ContainerDto, BaseErr>;
    pub fn async create_container(&self, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    pub fn async update_container(&self, id: &str, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    pub fn async delete_container(&self, id: &str) -> Result<(), BaseErr>;
}