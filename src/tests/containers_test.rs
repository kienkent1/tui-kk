use std::{collections::HashMap, sync::Arc};

use bollard::{
    Docker,
    plugin::{ContainerSummary, ContainerSummaryStateEnum, PortSummary, PortSummaryTypeEnum},
};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    modules::containers::{
        constants::{ContainerDetailTab, ContainerSort},
        container_dto::{container_display_name, format_ports, short_container_id},
        container_service::ContainerService,
        list_page::ContainerListPage,
    },
    shared::base::{base_component::Page, base_filter::BaseFilter},
    tuikk_core::{key_map::Command, themes::ThemeColor},
};

fn dummy_service() -> Arc<ContainerService> {
    let docker =
        Docker::connect_with_http("http://127.0.0.1:2375", 4, bollard::API_DEFAULT_VERSION)
            .unwrap();
    Arc::new(ContainerService::new(Arc::new(docker)))
}

fn sample_containers() -> Vec<ContainerSummary> {
    vec![
        ContainerSummary {
            id: Some("111111111111aaaa".to_owned()),
            names: Some(vec!["/web-nginx".to_owned()]),
            image: Some("nginx:alpine".to_owned()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            created: Some(200),
            ..Default::default()
        },
        ContainerSummary {
            id: Some("222222222222bbbb".to_owned()),
            names: Some(vec!["/api-backend".to_owned()]),
            image: Some("rust:1.88".to_owned()),
            state: Some(ContainerSummaryStateEnum::EXITED),
            created: Some(100),
            ..Default::default()
        },
        ContainerSummary {
            id: Some("333333333333cccc".to_owned()),
            names: Some(vec!["/db-postgres".to_owned()]),
            image: Some("postgres:16".to_owned()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            created: Some(300),
            ..Default::default()
        },
    ]
}

#[test]
fn container_sort_parse_and_cycle() {
    assert_eq!(
        ContainerSort::parse("name"),
        Some((ContainerSort::Name, false))
    );
    assert_eq!(
        ContainerSort::parse("-created"),
        Some((ContainerSort::Created, true))
    );
    assert_eq!(
        ContainerSort::parse(" status "),
        Some((ContainerSort::State, false))
    );
    assert_eq!(ContainerSort::parse("unknown"), None);

    assert_eq!(ContainerSort::Name.next(), ContainerSort::Image);
    assert_eq!(ContainerSort::Image.next(), ContainerSort::State);
    assert_eq!(ContainerSort::State.next(), ContainerSort::Created);
    assert_eq!(ContainerSort::Created.next(), ContainerSort::Name);

    assert_eq!(ContainerSort::Name.to_filter_string(false), "name");
    assert_eq!(ContainerSort::Created.to_filter_string(true), "-created");
}

#[test]
fn detail_tab_cycles_forward_and_backward() {
    let tab = ContainerDetailTab::Overview;
    assert_eq!(tab.next(), ContainerDetailTab::NetworkMounts);
    assert_eq!(tab.next().next(), ContainerDetailTab::Env);
    assert_eq!(tab.next().next().next(), ContainerDetailTab::RawJson);
    assert_eq!(
        tab.next().next().next().next(),
        ContainerDetailTab::Overview
    );
    assert_eq!(tab.prev(), ContainerDetailTab::RawJson);
}

#[test]
fn dto_format_helpers_deduplicate_ports_and_trim_names() {
    let c = ContainerSummary {
        id: Some("abcdef1234567890".to_owned()),
        names: Some(vec!["/my-container".to_owned()]),
        ..Default::default()
    };
    assert_eq!(short_container_id(&c), "abcdef123456");
    assert_eq!(container_display_name(&c), "my-container");

    let ports = vec![
        PortSummary {
            ip: Some("0.0.0.0".to_owned()),
            private_port: 80,
            public_port: Some(8080),
            typ: Some(PortSummaryTypeEnum::TCP),
        },
        // Duplicate IPv6 entry for same port mapping
        PortSummary {
            ip: Some("::".to_owned()),
            private_port: 80,
            public_port: Some(8080),
            typ: Some(PortSummaryTypeEnum::TCP),
        },
        PortSummary {
            ip: None,
            private_port: 53,
            public_port: None,
            typ: Some(PortSummaryTypeEnum::UDP),
        },
    ];
    assert_eq!(format_ports(Some(&ports)), "udp/53, tcp/8080:80");
}

#[test]
fn query_containers_search_filter_and_sort() {
    let svc = dummy_service();
    let items = sample_containers();

    // Search by partial image name (case-insensitive)
    let filter = BaseFilter {
        search: Some("POSTGRES".to_owned()),
        ..Default::default()
    };
    let res = svc.query_containers(&items, &filter);
    assert_eq!(res.len(), 1);
    assert_eq!(container_display_name(res[0]), "db-postgres");

    // Filter by status=running and sort by -created (descending)
    let mut map = HashMap::new();
    map.insert("status".to_owned(), vec!["running".to_owned()]);
    let filter = BaseFilter {
        sort: Some("-created".to_owned()),
        filters: Some(map),
        ..Default::default()
    };
    let res = svc.query_containers(&items, &filter);
    assert_eq!(res.len(), 2);
    assert_eq!(container_display_name(res[0]), "db-postgres"); // created 300
    assert_eq!(container_display_name(res[1]), "web-nginx"); // created 200
}

#[test]
fn list_page_cursor_clamping_and_commands() {
    let svc = dummy_service();
    let (tx, _rx) = unbounded_channel();
    let mut page = ContainerListPage::new(svc, tx, true, &ThemeColor::dark());
    page.set_viewport(2);
    page.set_items(sample_containers());

    assert_eq!(page.cursor(), 0);
    assert_eq!(page.offset(), 0);
    assert_eq!(page.selected_id(), Some("111111111111aaaa"));

    // Move down twice -> viewport is 2, so offset shifts to 1
    page.handle_command(Command::Down);
    assert_eq!(page.cursor(), 1);
    assert_eq!(page.offset(), 0);

    page.handle_command(Command::Down);
    assert_eq!(page.cursor(), 2);
    assert_eq!(page.offset(), 1);
    assert_eq!(page.selected_id(), Some("333333333333cccc"));

    // Moving past the end clamps at last element
    page.handle_command(Command::Down);
    assert_eq!(page.cursor(), 2);

    // Cycle sort by Name -> api-backend, db-postgres, web-nginx
    page.cycle_sort();
    let names: Vec<&str> = page
        .rows()
        .iter()
        .map(|c| container_display_name(c))
        .collect();
    assert_eq!(names, vec!["api-backend", "db-postgres", "web-nginx"]);
}
