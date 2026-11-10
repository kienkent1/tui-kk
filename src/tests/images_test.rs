use std::{collections::HashMap, sync::Arc};

use bollard::{Docker, models::ImageSummary};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    modules::images::{
        constants::{ImageDetailTab, ImageSort},
        image_dto::{format_image_size, parse_repo_tag, short_image_id},
        image_service::ImageService,
        list_page::ImageListPage,
    },
    shared::base::{base_component::Page, base_filter::BaseFilter},
    tuikk_core::{key_map::Command, themes::ThemeColor},
};

fn dummy_image_service() -> Arc<ImageService> {
    let docker = Docker::connect_with_local_defaults().unwrap();
    Arc::new(ImageService::new(Arc::new(docker)))
}

fn sample_images() -> Vec<ImageSummary> {
    vec![
        ImageSummary {
            id: "sha256:aaaa1111222233334444".to_owned(),
            parent_id: String::new(),
            repo_tags: vec!["nginx:alpine".to_owned()],
            repo_digests: Vec::new(),
            created: 2000,
            size: 40 * 1024 * 1024,
            shared_size: 0,
            labels: HashMap::new(),
            containers: 2,
            manifests: None,
            descriptor: None,
        },
        ImageSummary {
            id: "sha256:bbbb1111222233334444".to_owned(),
            parent_id: String::new(),
            repo_tags: vec!["localhost:5000/org/api:v1.2".to_owned()],
            repo_digests: Vec::new(),
            created: 3000,
            size: 120 * 1024 * 1024,
            shared_size: 0,
            labels: HashMap::new(),
            containers: 0,
            manifests: None,
            descriptor: None,
        },
        ImageSummary {
            id: "sha256:cccc1111222233334444".to_owned(),
            parent_id: String::new(),
            repo_tags: Vec::new(), // untagged / dangling
            repo_digests: Vec::new(),
            created: 1000,
            size: 10 * 1024 * 1024,
            shared_size: 0,
            labels: HashMap::new(),
            containers: 1,
            manifests: None,
            descriptor: None,
        },
    ]
}

#[test]
fn image_sort_parse_and_cycle() {
    assert_eq!(ImageSort::parse("repo"), Some((ImageSort::RepoTag, false)));
    assert_eq!(ImageSort::parse("-size"), Some((ImageSort::Size, true)));
    assert_eq!(
        ImageSort::parse("created"),
        Some((ImageSort::Created, false))
    );
    assert_eq!(
        ImageSort::parse("-containers"),
        Some((ImageSort::Containers, true))
    );
    assert_eq!(ImageSort::parse("invalid"), None);

    assert_eq!(ImageSort::RepoTag.next(), ImageSort::Id);
    assert_eq!(ImageSort::Id.next(), ImageSort::Size);
    assert_eq!(ImageSort::Size.next(), ImageSort::Created);
    assert_eq!(ImageSort::Created.next(), ImageSort::Containers);
    assert_eq!(ImageSort::Containers.next(), ImageSort::RepoTag);
}

#[test]
fn image_detail_tab_cycles() {
    let tab = ImageDetailTab::Overview;
    assert_eq!(tab.next(), ImageDetailTab::Layers);
    assert_eq!(tab.next().next(), ImageDetailTab::Config);
    assert_eq!(tab.next().next().next(), ImageDetailTab::RawJson);
    assert_eq!(tab.next().next().next().next(), ImageDetailTab::Overview);
    assert_eq!(tab.prev(), ImageDetailTab::RawJson);
}

#[test]
fn image_dto_helpers_parse_repo_tag_and_short_id() {
    assert_eq!(short_image_id("sha256:abcdef1234567890"), "abcdef123456");
    assert_eq!(short_image_id("12345"), "12345");
    assert_eq!(format_image_size(0), "0 B");
    assert_eq!(format_image_size(-1), "0 B");
    assert_eq!(format_image_size(10 * 1024 * 1024), "10.0 MB");

    let items = sample_images();
    assert_eq!(
        parse_repo_tag(&items[0]),
        ("nginx".to_owned(), "alpine".to_owned())
    );
    // Registry host with port + tag
    assert_eq!(
        parse_repo_tag(&items[1]),
        ("localhost:5000/org/api".to_owned(), "v1.2".to_owned())
    );
    // Untagged image
    assert_eq!(
        parse_repo_tag(&items[2]),
        ("<none>".to_owned(), "<none>".to_owned())
    );
}

#[test]
fn query_images_search_filter_and_sort() {
    let svc = dummy_image_service();
    let items = sample_images();

    // Search by tag/repository
    let filter = BaseFilter {
        search: Some("ALPINE".to_owned()),
        ..Default::default()
    };
    let res = svc.query_images(&items, &filter);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "sha256:aaaa1111222233334444");

    // Filter dangling=true
    let mut map = HashMap::new();
    map.insert("dangling".to_owned(), vec!["true".to_owned()]);
    let filter = BaseFilter {
        filters: Some(map),
        ..Default::default()
    };
    let res = svc.query_images(&items, &filter);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "sha256:cccc1111222233334444");

    // Sort by -size (descending)
    let filter = BaseFilter {
        sort: Some("-size".to_owned()),
        ..Default::default()
    };
    let res = svc.query_images(&items, &filter);
    assert_eq!(res[0].id, "sha256:bbbb1111222233334444");
    assert_eq!(res[1].id, "sha256:aaaa1111222233334444");
    assert_eq!(res[2].id, "sha256:cccc1111222233334444");
}

#[test]
fn image_list_page_navigation_and_sorting() {
    let svc = dummy_image_service();
    let (tx, _rx) = unbounded_channel();
    let mut page = ImageListPage::new(svc, tx, true, &ThemeColor::dark());
    page.set_viewport(2);
    page.set_items(sample_images());

    assert_eq!(page.cursor(), 0);
    assert_eq!(page.offset(), 0);
    assert_eq!(page.selected_id(), Some("sha256:aaaa1111222233334444"));

    page.handle_command(Command::Down);
    page.handle_command(Command::Down);
    assert_eq!(page.cursor(), 2);
    assert_eq!(page.offset(), 1);
    assert_eq!(page.selected_id(), Some("sha256:cccc1111222233334444"));

    // Sort by RepoTag ascending: "<none>", "localhost:5000/org/api", "nginx"
    page.cycle_sort();
    let ids: Vec<&str> = page.rows().iter().map(|img| img.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "sha256:cccc1111222233334444",
            "sha256:bbbb1111222233334444",
            "sha256:aaaa1111222233334444",
        ]
    );
}
