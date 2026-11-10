use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use bollard::{
    Docker,
    models::{ImageInspect, ImageSummary},
    query_parameters::{ListImagesOptions, PruneImagesOptions, RemoveImageOptions},
};

use crate::{
    modules::images::{constants::ImageSort, image_dto::parse_repo_tag},
    shared::{
        base::{base_err::BaseErr, base_filter::BaseFilter},
        helpers::ext_log::ResultExt,
    },
};

pub struct ImageService {
    conn: Arc<Docker>,
}

impl ImageService {
    pub fn new(conn: Arc<Docker>) -> Self {
        Self { conn }
    }

    pub async fn fetch_images(
        &self,
        filter: &BaseFilter,
        all: Option<bool>,
    ) -> Result<Vec<ImageSummary>, BaseErr> {
        let query = ListImagesOptions {
            all: all.unwrap_or(false),
            filters: filter.filters.clone(),
            shared_size: false,
            digests: false,
            manifests: false,
        };
        Ok(self.conn.list_images(Some(query)).await?)
    }

    pub async fn get_images(
        &self,
        filter: BaseFilter,
        all: Option<bool>,
    ) -> Result<Vec<ImageSummary>, BaseErr> {
        let mut items = self.fetch_images(&filter, all).await?;
        let needle = Self::needle(&filter);
        let sort = Self::parse_sort(&filter);
        items.retain(|img| needle.is_empty() || self.match_search(img, &needle));
        items.sort_by(|a, b| Self::compare(sort, a, b));
        Ok(items)
    }

    pub fn query_images<'a>(
        &self,
        items: &'a [ImageSummary],
        filter: &BaseFilter,
    ) -> Vec<&'a ImageSummary> {
        let needle = Self::needle(filter);
        let sort = Self::parse_sort(filter);
        let empty = HashMap::new();
        let filters = filter.filters.as_ref().unwrap_or(&empty);

        let mut v: Vec<&ImageSummary> = items
            .iter()
            .filter(|img| Self::match_filters(img, filters))
            .filter(|img| needle.is_empty() || self.match_search(img, &needle))
            .collect();
        v.sort_by(|a, b| Self::compare(sort, a, b));
        v
    }

    pub async fn get_image(&self, id: &str) -> Result<ImageInspect, BaseErr> {
        self.conn
            .inspect_image(id)
            .await
            .map_err(|e| match BaseErr::from(e) {
                BaseErr::NotFound { .. } => BaseErr::NotFound {
                    resource_type: "Image",
                    id: id.to_owned(),
                },
                other => other,
            })
            .log_err()
    }

    pub async fn remove_image(&self, id: &str, force: bool) -> Result<(), BaseErr> {
        let opts = RemoveImageOptions {
            force,
            noprune: false,
            platforms: None,
        };
        self.conn
            .remove_image(id, Some(opts), None)
            .await
            .map(|_| ())
            .map_err(BaseErr::from)
            .log_err()
    }

    pub async fn prune_images(&self) -> Result<u64, BaseErr> {
        let opts = PruneImagesOptions { filters: None };
        let res = self
            .conn
            .prune_images(Some(opts))
            .await
            .map_err(BaseErr::from)
            .log_err()?;
        Ok(res.space_reclaimed.unwrap_or(0).max(0) as u64)
    }
}

impl ImageService {
    fn contains_ci(&self, v: &str, search: &str) -> bool {
        v.to_lowercase().contains(search)
    }

    fn match_search(&self, img: &ImageSummary, search: &str) -> bool {
        img.repo_tags.iter().any(|t| self.contains_ci(t, search))
            || self.contains_ci(&img.id, search)
    }

    fn needle(f: &BaseFilter) -> String {
        f.search
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_lowercase()
    }

    fn parse_sort(f: &BaseFilter) -> Option<(ImageSort, bool)> {
        f.sort
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(ImageSort::parse)
    }

    fn compare(sort: Option<(ImageSort, bool)>, a: &ImageSummary, b: &ImageSummary) -> Ordering {
        let Some((key, desc)) = sort else {
            return Ordering::Equal;
        };
        let ord = match key {
            ImageSort::RepoTag => {
                let (ra, ta) = parse_repo_tag(a);
                let (rb, tb) = parse_repo_tag(b);
                ra.cmp(&rb).then_with(|| ta.cmp(&tb))
            }
            ImageSort::Id => a.id.cmp(&b.id),
            ImageSort::Size => a.size.cmp(&b.size),
            ImageSort::Created => a.created.cmp(&b.created),
            ImageSort::Containers => a.containers.cmp(&b.containers),
        };
        if desc { ord.reverse() } else { ord }
    }

    fn match_filters(img: &ImageSummary, filters: &HashMap<String, Vec<String>>) -> bool {
        filters.iter().all(|(k, v)| {
            if v.is_empty() {
                return true;
            }
            match k.as_str() {
                "reference" | "repo" => img
                    .repo_tags
                    .iter()
                    .any(|tag| v.iter().any(|pat| tag.contains(pat))),
                "dangling" => {
                    let is_dangling = img.repo_tags.is_empty()
                        || img.repo_tags.iter().all(|t| t == "<none>:<none>");
                    let want_dangling = v.iter().any(|x| x == "true");
                    is_dangling == want_dangling
                }
                _ => true,
            }
        })
    }
}
