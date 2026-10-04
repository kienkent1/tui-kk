use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use bollard::{
    Docker,
    plugin::{ContainerInspectResponse, ContainerSummary, ContainerSummaryStateEnum},
    query_parameters::{InspectContainerOptions, ListContainersOptions},
};

use crate::{
    modules::containers::constants::ContainerSort,
    shared::{
        base::{base_err::BaseErr, base_filter::BaseFilter},
        helpers::ext_log::ResultExt,
    },
};

pub struct ContainerService {
    conn: Arc<Docker>,
}

//Call bollard API to get containers, inspect container, create, update, delete container
impl ContainerService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }

    pub async fn fetch_containers(
        &self,
        filter: &BaseFilter,
        all: Option<bool>,
        size: Option<bool>,
    ) -> Result<Vec<ContainerSummary>, BaseErr> {
        let query = ListContainersOptions {
            all: all.unwrap_or(true),
            limit: filter.limit.filter(|&n| n > 0),
            size: size.unwrap_or(false),
            filters: filter.filters.clone(),
        };
        Ok(self.conn.list_containers(Some(query)).await?)
    }

    pub async fn get_containers(
        &self,
        filter: BaseFilter,
        all: Option<bool>,
    ) -> Result<Vec<ContainerSummary>, BaseErr> {
        let mut items = self.fetch_containers(&filter, all, None).await?;
        let needle = Self::needle(&filter);
        let sort = Self::parse_sort(&filter);
        items.retain(|c| needle.is_empty() || self.match_search(c, &needle));
        items.sort_by(|a, b| Self::compare(sort, a, b));
        Ok(items)
    }

    pub fn query_containers<'a>(
        &self,
        items: &'a [ContainerSummary],
        filter: &BaseFilter,
    ) -> Vec<&'a ContainerSummary> {
        let needle = Self::needle(filter);
        let sort = Self::parse_sort(filter);
        let empty = HashMap::new();
        let filters = filter.filters.as_ref().unwrap_or(&empty);

        let mut v: Vec<&ContainerSummary> = items
            .iter()
            .filter(|c| Self::match_filters(c, filters))
            .filter(|c| needle.is_empty() || self.match_search(c, &needle))
            .collect();
        v.sort_by(|a, b| Self::compare(sort, a, b));
        v
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
}

//Helper functions for ContainerService
impl ContainerService {
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

    fn needle(f: &BaseFilter) -> String {
        f.search
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_lowercase()
    }

    fn parse_sort(f: &BaseFilter) -> Option<(ContainerSort, bool)> {
        f.sort
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(ContainerSort::parse)
    }

    fn compare(
        sort: Option<(ContainerSort, bool)>,
        a: &ContainerSummary,
        b: &ContainerSummary,
    ) -> Ordering {
        let Some((key, asc)) = sort else {
            return Ordering::Equal;
        };
        let ord = match key {
            ContainerSort::Name => Self::first_name(a).cmp(Self::first_name(b)),
            ContainerSort::Image => a.image.cmp(&b.image),
            ContainerSort::State => a.state.cmp(&b.state),
            ContainerSort::Created => a.created.cmp(&b.created),
        };
        if asc { ord } else { ord.reverse() }
    }

    fn first_name(c: &ContainerSummary) -> &str {
        c.names
            .as_ref()
            .and_then(|n| n.first())
            .map(|s| s.trim_start_matches('/'))
            .unwrap_or("")
    }

    fn match_filters(c: &ContainerSummary, filters: &HashMap<String, Vec<String>>) -> bool {
        filters.iter().all(|(k, v)| {
            if v.is_empty() {
                return true;
            }
            match k.as_str() {
                "name" => c.names.iter().flatten().any(|n| {
                    let n = n.trim_start_matches('/');
                    v.iter().any(|x| x.trim_start_matches('/') == n)
                }),
                "ancestor" | "image" => c.image.as_ref().is_some_and(|i| v.contains(i)),
                // Docker: status = state (running, exited, ...)
                "status" | "state" => c.state.is_some_and(|s| {
                    let s = s.to_string();
                    v.iter().any(|x| *x == s)
                }),
                _ => true,
            }
        })
    }
}
