use std::sync::Arc;

use bollard::{Docker, plugin::ContainerSummary, query_parameters::ListContainersOptionsBuilder};

use crate::{modules::containers::container_dto::ContainerDto, shared::base::{base_err::BaseErr, base_filter::BaseFilter}};

pub struct ContainerService {
    conn: Arc<Docker>,
}

impl ContainerService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }
    pub async fn get_containers(&self, filter: BaseFilter, all: Option<bool>, size: Option<bool>) -> Result<Vec<ContainerSummary>, BaseErr> {
        let all = all.unwrap_or(true);
        let size = size.unwrap_or(false);
        let mut query = ListContainersOptionsBuilder::new().all(all).limit(filter.limit).size(size).filters();
        let containers = self.conn.list_containers(Some(query.build())).await;
        containers?
    }

    // pub fn async get_container(&self, id: &str) -> Result<ContainerDto, BaseErr>;
    // pub fn async create_container(&self, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    // pub fn async update_container(&self, id: &str, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    // pub fn async delete_container(&self, id: &str) -> Result<(), BaseErr>;
}