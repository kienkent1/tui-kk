use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    modules::dashboard::dashboard_dto::{DashboardSnapshot, format_bytes},
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let [engine_area, containers_area, storage_area] = Layout::horizontal([
        Constraint::Percentage(34),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .areas(area);

    draw_engine_card(frame, engine_area, snapshot, loading, error, theme);
    draw_containers_card(frame, containers_area, snapshot, theme);
    draw_disk_card(frame, storage_area, snapshot, theme);
}

fn draw_engine_card(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let mut title = String::from(" Engine Overview ");
    if loading {
        title.push_str("· syncing… ");
    }
    if error.is_some() {
        title.push_str("· error ");
    }

    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let ver = if snapshot.engine_version.is_empty() {
        "-"
    } else {
        &snapshot.engine_version
    };
    let os = if snapshot.os_arch.is_empty() {
        "-"
    } else {
        &snapshot.os_arch
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(" Version: ", Style::new().fg(theme.text_muted)),
            Span::styled(ver, Style::new().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("  OS: ", Style::new().fg(theme.text_muted)),
            Span::styled(os, Style::new().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" Cores:   ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format!("{} vCPU", snapshot.ncpu.max(1)),
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  RAM: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format_bytes(snapshot.host_mem_total),
                Style::new().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_containers_card(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    theme: &ThemeColor,
) {
    let block = Block::bordered()
        .title(format!(" Containers ({}) ", snapshot.containers_total))
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let lines = vec![
        Line::from(vec![
            Span::styled(" ● Running: ", Style::new().fg(theme.status_running)),
            Span::styled(
                snapshot.containers_running.to_string(),
                Style::new()
                    .fg(theme.status_running)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ⏸ Paused: ", Style::new().fg(theme.status_paused)),
            Span::styled(
                snapshot.containers_paused.to_string(),
                Style::new()
                    .fg(theme.status_paused)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" ■ Stopped: ", Style::new().fg(theme.status_stopped)),
            Span::styled(
                snapshot.containers_stopped.to_string(),
                Style::new()
                    .fg(theme.status_stopped)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Images:   ", Style::new().fg(theme.text_muted)),
            Span::styled(
                snapshot.images_total.to_string(),
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_disk_card(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    theme: &ThemeColor,
) {
    let st = &snapshot.storage;
    let block = Block::bordered()
        .title(format!(
            " Disk Usage ({}) ",
            format_bytes(st.total_used())
        ))
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let lines = vec![
        Line::from(vec![
            Span::styled(" Images:     ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format_bytes(st.images_total_size),
                Style::new().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Volumes: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format_bytes(st.volumes_total_size),
                Style::new().fg(theme.warning).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Containers: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format_bytes(st.containers_total_size),
                Style::new().fg(theme.success).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Reclaim: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                format_bytes(st.images_reclaimable + st.containers_reclaimable),
                Style::new().fg(theme.text),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

