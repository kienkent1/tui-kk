use std::sync::Arc;

use bollard::{
    Docker,
    plugin::{ContainerInspectResponse, ContainerSummary},
    query_parameters::{
        InspectContainerOptions, ListContainersOptions, ListContainersOptionsBuilder,
    },
};

use crate::{
    modules::containers::{constants::ContainerSort, container_dto::ContainerDto},
    shared::{
        base::{base_err::BaseErr, base_filter::BaseFilter},
        helpers::ext_log::ResultExt,
    },
};

pub struct ContainerService {
    conn: Arc<Docker>,
}

impl ContainerService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }
    pub async fn get_containers(
        &self,
        filter: BaseFilter,
        all: Option<bool>,
        size: Option<bool>,
    ) -> Result<Vec<ContainerSummary>, BaseErr> {
        let all = all.unwrap_or(true);
        let size = size.unwrap_or(false);
        let limit = filter.limit.filter(|&n| n > 0);
        let query = ListContainersOptions {
            all: all,
            limit: limit,
            size: size,
            filters: filter.filters,
        };

        let mut containers = self.conn.list_containers(Some(query)).await?;

        if let Some(s) = filter
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let search = s.to_lowercase();
            containers.retain(|c| self.match_search(c, &search));
        }

        if let Some(sort) = filter.sort.as_deref().filter(|s| !s.is_empty()) {
            if let Some((key, order)) = ContainerSort::parse(sort) {
                self.sort_containers(&mut containers, key, order);
            }
        }

        Ok(containers)
    }

    pub async fn get_container(&self, name: &str) -> Result<ContainerInspectResponse, BaseErr> {
        let options = InspectContainerOptions { size: true };
        self.conn
            .inspect_container(name, Some(options))
            .await
            .map_err(|e| match BaseErr::from(e) {
                BaseErr::NotFound { .. } => BaseErr::NotFound {
                    resource_type: "Container",
                    id: name.to_owned(),
                },
                other => other,
            })
            .log_err()
    }
    // pub fn async create_container(&self, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    // pub fn async update_container(&self, id: &str, container: &ContainerDto) -> Result<ContainerDto, BaseErr>;
    // pub fn async delete_container(&self, id: &str) -> Result<(), BaseErr>;

    ///Helper fn
    fn contains_ci(&self, v: &str, search: &str) -> bool {
        v.to_lowercase().contains(search)
    }
    fn match_search(&self, c: &ContainerSummary, search: &str) -> bool {
        c.names
            .iter()
            .flatten()
            .any(|n| self.contains_ci(n, search))
            || c.image
                .as_deref()
                .is_some_and(|v| self.contains_ci(v, search))
            || c.id.as_deref().is_some_and(|v| self.contains_ci(v, search))
    }

    fn sort_containers(&self, list: &mut [ContainerSummary], key: ContainerSort, order: bool) {
        match key {
            ContainerSort::Name => list.sort_by_cached_key(|c| c.names.clone()),
            ContainerSort::Image => list.sort_by_key(|c| c.image.clone()),
            ContainerSort::State => list.sort_by_key(|c| c.state),
            ContainerSort::Created => list.sort_by_key(|c| c.created),
        }
        if !order {
            list.reverse();
        }
    }
}
