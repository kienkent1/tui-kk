use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use bollard::{Docker, models::ImageSummary};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    modules::images::{
        constants::{ImageDetailTab, ImageSort, RunField},
        image_dto::{
            EnvVarEntry, RunImageRequest, format_image_size, merge_env_vars, parse_env_content,
            parse_env_file, parse_port_mappings, parse_repo_tag, parse_volume_binds,
            primary_image_ref, short_image_id,
        },
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
fn image_detail_tab_and_run_field_cycle() {
    let tab = ImageDetailTab::Overview;
    assert_eq!(tab.next(), ImageDetailTab::Layers);
    assert_eq!(tab.next().next(), ImageDetailTab::Config);
    assert_eq!(tab.next().next().next(), ImageDetailTab::RawJson);
    assert_eq!(tab.next().next().next().next(), ImageDetailTab::Overview);
    assert_eq!(tab.prev(), ImageDetailTab::RawJson);

    let f = RunField::ContainerName;
    assert_eq!(f.next(), RunField::Ports);
    assert_eq!(f.prev(), RunField::RunButton);
    assert_eq!(RunField::RunButton.next(), RunField::ContainerName);
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
    assert_eq!(primary_image_ref(&items[0]), "nginx:alpine");
    // Registry host with port + tag
    assert_eq!(
        parse_repo_tag(&items[1]),
        ("localhost:5000/org/api".to_owned(), "v1.2".to_owned())
    );
    assert_eq!(primary_image_ref(&items[1]), "localhost:5000/org/api:v1.2");
    // Untagged image
    assert_eq!(
        parse_repo_tag(&items[2]),
        ("<none>".to_owned(), "<none>".to_owned())
    );
    assert_eq!(primary_image_ref(&items[2]), "cccc11112222");
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

#[test]
fn parse_env_content_and_file_handles_quotes_exports_and_comments() {
    let content = r#"
# Database configuration
DB_HOST = localhost # inline comment
export DB_PORT=5432
DB_PASSWORD="my \"secret\"\npass"
JWT_SECRET='single_quoted_#not_a_comment'
EMPTY_VAL=
DB_PORT=6432
"#;

    let vars = parse_env_content(content);
    assert_eq!(vars.len(), 5);
    assert_eq!(vars[0], EnvVarEntry::new("DB_HOST", "localhost", ".env"));
    // Duplicate key DB_PORT updated in place to 6432
    assert_eq!(vars[1], EnvVarEntry::new("DB_PORT", "6432", ".env"));
    assert_eq!(
        vars[2],
        EnvVarEntry::new("DB_PASSWORD", "my \"secret\"\npass", ".env")
    );
    assert_eq!(
        vars[3],
        EnvVarEntry::new("JWT_SECRET", "single_quoted_#not_a_comment", ".env")
    );
    assert_eq!(vars[4], EnvVarEntry::new("EMPTY_VAL", "", ".env"));

    // Test reading from a temporary .env file on disk
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_path: PathBuf = std::env::temp_dir().join(format!("tuikk_test_{nanos}.env"));
    fs::write(&tmp_path, "APP_ENV=production\nPORT=8080\n").unwrap();

    let from_file = parse_env_file(&tmp_path).unwrap();
    assert_eq!(from_file.len(), 2);
    assert_eq!(from_file[0].key, "APP_ENV");
    assert_eq!(from_file[0].value, "production");
    assert_eq!(from_file[1].key, "PORT");
    assert_eq!(from_file[1].value, "8080");

    let _ = fs::remove_file(&tmp_path);
}

#[test]
fn merge_env_ports_volumes_and_build_container_body() {
    let mut vars = vec![
        EnvVarEntry::new("A", "1", ".env"),
        EnvVarEntry::new("B", "2", ".env"),
    ];
    merge_env_vars(
        &mut vars,
        vec![
            EnvVarEntry::new("B", "override", "manual"),
            EnvVarEntry::new("C", "3", "manual"),
        ],
    );
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[1].value, "override");
    assert_eq!(vars[1].source, "manual");
    assert_eq!(vars[2].key, "C");

    let ports = parse_port_mappings("8080:80, 5432:5432/tcp 5353:53/udp");
    assert_eq!(
        ports,
        vec![
            ("8080".to_owned(), "80/tcp".to_owned()),
            ("5432".to_owned(), "5432/tcp".to_owned()),
            ("5353".to_owned(), "53/udp".to_owned()),
        ]
    );

    let volumes = parse_volume_binds("/host/data:/data, my_vol:/var/lib/postgresql/data:ro");
    assert_eq!(
        volumes,
        vec![
            "/host/data:/data".to_owned(),
            "my_vol:/var/lib/postgresql/data:ro".to_owned(),
        ]
    );

    let req = RunImageRequest {
        image: "postgres:16".to_owned(),
        container_name: Some("pg-dev".to_owned()),
        ports,
        volumes,
        env_vars: vars,
    };

    let body = ImageService::build_container_create_body(&req);
    assert_eq!(body.image.as_deref(), Some("postgres:16"));
    assert_eq!(
        body.env,
        Some(vec![
            "A=1".to_owned(),
            "B=override".to_owned(),
            "C=3".to_owned(),
        ])
    );
    assert_eq!(
        body.exposed_ports,
        Some(vec![
            "80/tcp".to_owned(),
            "5432/tcp".to_owned(),
            "53/udp".to_owned(),
        ])
    );

    let hc = body.host_config.expect("host_config should be populated");
    assert_eq!(hc.binds.as_ref().map(|v| v.len()), Some(2));
    let pb = hc.port_bindings.expect("port_bindings should be populated");
    assert_eq!(
        pb.get("80/tcp")
            .and_then(|v| v.as_ref())
            .and_then(|v| v.first())
            .and_then(|b| b.host_port.as_deref()),
        Some("8080")
    );
}
