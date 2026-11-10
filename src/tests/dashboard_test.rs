use std::collections::HashMap;

use bollard::models::{
    ContainerCpuStats, ContainerCpuUsage, ContainerMemoryStats, ContainerStatsResponse,
    ContainerSummary, ContainerSummaryStateEnum, ContainersDiskUsage, ImagesDiskUsage,
    SystemDataUsageResponse, SystemInfo, VolumesDiskUsage,
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::{
    modules::dashboard::{
        constants::{GB, KB, MB, TB},
        dashboard_dto::{MetricHistory, format_bytes},
        dashboard_service::DashboardService,
    },
    shared::{
        base::base_widget::BaseWidget,
        widgets::side_bar::{SideBar, SideBarEvent},
    },
    tuikk_core::{route::Route, themes::ThemeColor},
};

#[test]
fn format_bytes_units() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(KB as u64), "1.0 KB");
    assert_eq!(format_bytes((MB * 2.5) as u64), "2.5 MB");
    assert_eq!(format_bytes((GB * 1.25) as u64), "1.25 GB");
    assert_eq!(format_bytes((TB * 3.0) as u64), "3.00 TB");
}

#[test]
fn metric_history_ring_buffer_and_clamping() {
    let mut h = MetricHistory::new(3);
    h.push(10.0, 20.0, 100.0);
    h.push(50.0, 60.0, 200.0);
    h.push(120.0, -5.0, -10.0); // clamped to 100.0, 0.0, 0.0

    assert_eq!(h.cpu_dataset(), vec![(0.0, 10.0), (1.0, 50.0), (2.0, 100.0)]);
    assert_eq!(h.mem_dataset(), vec![(0.0, 20.0), (1.0, 60.0), (2.0, 0.0)]);
    assert_eq!(
        h.storage_dataset(),
        vec![(0.0, 100.0), (1.0, 200.0), (2.0, 0.0)]
    );

    // Pushing a 4th item evicts the oldest
    h.push(30.0, 40.0, 300.0);
    assert_eq!(h.cpu_dataset(), vec![(0.0, 50.0), (1.0, 100.0), (2.0, 30.0)]);
}

#[test]
fn calculate_cpu_percent_multi_core() {
    let stats = ContainerStatsResponse {
        cpu_stats: Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                total_usage: Some(250_000_000),
                ..Default::default()
            }),
            system_cpu_usage: Some(2_000_000_000),
            online_cpus: Some(4),
            ..Default::default()
        }),
        precpu_stats: Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                total_usage: Some(150_000_000),
                ..Default::default()
            }),
            system_cpu_usage: Some(1_000_000_000),
            online_cpus: Some(4),
            ..Default::default()
        }),
        ..Default::default()
    };

    // cpu_delta = 100_000_000, sys_delta = 1_000_000_000, online_cpus = 4 -> 40.0%
    let pct = DashboardService::calculate_cpu_percent(&stats);
    assert!((pct - 40.0).abs() < 1e-6);
}

#[test]
fn calculate_cpu_percent_zero_or_missing_returns_zero() {
    let empty = ContainerStatsResponse::default();
    assert_eq!(DashboardService::calculate_cpu_percent(&empty), 0.0);

    let zero_delta = ContainerStatsResponse {
        cpu_stats: Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                total_usage: Some(100),
                ..Default::default()
            }),
            system_cpu_usage: Some(1000),
            online_cpus: Some(2),
            ..Default::default()
        }),
        precpu_stats: Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                total_usage: Some(100),
                ..Default::default()
            }),
            system_cpu_usage: Some(1000),
            online_cpus: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(DashboardService::calculate_cpu_percent(&zero_delta), 0.0);
}

#[test]
fn calculate_mem_usage_subtracts_inactive_file_cache() {
    let mut mem_stats_map = HashMap::new();
    mem_stats_map.insert("inactive_file".to_owned(), 50 * 1024 * 1024);

    let stats = ContainerStatsResponse {
        memory_stats: Some(ContainerMemoryStats {
            usage: Some(250 * 1024 * 1024),
            limit: Some(1000 * 1024 * 1024),
            stats: Some(mem_stats_map),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (used, limit, pct) = DashboardService::calculate_mem_usage(&stats, 0);
    assert_eq!(used, 200 * 1024 * 1024);
    assert_eq!(limit, 1000 * 1024 * 1024);
    assert!((pct - 20.0).abs() < 1e-6);
}

#[test]
fn build_snapshot_aggregates_engine_and_container_metrics() {
    let info = SystemInfo {
        server_version: Some("27.0.3".to_owned()),
        operating_system: Some("Ubuntu".to_owned()),
        architecture: Some("x86_64".to_owned()),
        ncpu: Some(2),
        mem_total: Some(1000),
        containers: Some(2),
        containers_running: Some(1),
        containers_stopped: Some(1),
        containers_paused: Some(0),
        images: Some(5),
        ..Default::default()
    };

    let df = SystemDataUsageResponse {
        image_usage: Some(ImagesDiskUsage {
            total_size: Some(5000),
            reclaimable: Some(1000),
            ..Default::default()
        }),
        container_usage: Some(ContainersDiskUsage {
            total_size: Some(2000),
            reclaimable: Some(500),
            ..Default::default()
        }),
        volume_usage: Some(VolumesDiskUsage {
            total_size: Some(3000),
            ..Default::default()
        }),
        ..Default::default()
    };

    let summaries = vec![
        ContainerSummary {
            id: Some("abcdef1234567890".to_owned()),
            names: Some(vec!["/redis-cache".to_owned()]),
            image: Some("redis:7".to_owned()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            size_rw: Some(120),
            size_root_fs: Some(800),
            ..Default::default()
        },
        ContainerSummary {
            id: Some("9876543210fedcba".to_owned()),
            names: Some(vec!["/old-worker".to_owned()]),
            image: Some("worker:1".to_owned()),
            state: Some(ContainerSummaryStateEnum::EXITED),
            size_rw: Some(50),
            size_root_fs: Some(600),
            ..Default::default()
        },
    ];

    let mut stats_map = HashMap::new();
    stats_map.insert(
        "abcdef1234567890".to_owned(),
        ContainerStatsResponse {
            cpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    total_usage: Some(200),
                    ..Default::default()
                }),
                system_cpu_usage: Some(1000),
                online_cpus: Some(2),
                ..Default::default()
            }),
            precpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    total_usage: Some(100),
                    ..Default::default()
                }),
                system_cpu_usage: Some(500),
                online_cpus: Some(2),
                ..Default::default()
            }),
            memory_stats: Some(ContainerMemoryStats {
                usage: Some(400),
                limit: Some(1000),
                ..Default::default()
            }),
            ..Default::default()
        },
    );

    let snap = DashboardService::build_snapshot(&info, &df, &summaries, &stats_map);
    assert_eq!(snap.engine_version, "27.0.3");
    assert_eq!(snap.os_arch, "Ubuntu (x86_64)");
    assert_eq!(snap.containers_total, 2);
    assert_eq!(snap.containers_running, 1);
    assert_eq!(snap.images_total, 5);
    assert_eq!(snap.storage.total_used(), 10_000);
    assert_eq!(snap.containers.len(), 2);
    assert_eq!(snap.containers[0].name, "redis-cache");
    assert!((snap.containers[0].cpu_percent - 40.0).abs() < 1e-5);
    assert!((snap.total_cpu_percent - 20.0).abs() < 1e-5);
    assert_eq!(snap.total_mem_usage, 400);
    assert!((snap.total_mem_percent - 40.0).abs() < 1e-5);
}

#[test]
fn sidebar_navigation_and_mouse_click() {
    let mut sb = SideBar::new(ThemeColor::dark());
    assert_eq!(sb.active_route(), Route::Dashboard);
    assert_eq!(sb.next_route(), Route::Containers);
    assert_eq!(sb.prev_route(), Route::Images);

    sb.set_active_route(Route::Containers);
    assert_eq!(sb.active_route(), Route::Containers);

    let area = Rect::new(0, 0, 22, 15);
    // Border takes 1 row, header takes 2 rows -> item 0 is at y = 3, item 1 at y = 4
    let click_containers = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 4,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };
    let ev = sb.handle_mouse(area, click_containers);
    assert_eq!(ev, Some(SideBarEvent::NavigatePage(Route::Containers)));
}

