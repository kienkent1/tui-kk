use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Cell, Row, Table, TableState},
};

use crate::{
    modules::dashboard::{
        constants::MB,
        dashboard_dto::{DashboardSnapshot, format_bytes},
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    cursor: usize,
    offset: usize,
    viewport: usize,
    theme: &ThemeColor,
) {
    let [bar_area, table_area] =
        Layout::horizontal([Constraint::Percentage(32), Constraint::Percentage(68)]).areas(area);

    draw_storage_barchart(frame, bar_area, snapshot, theme);
    draw_container_resource_table(frame, table_area, snapshot, cursor, offset, viewport, theme);
}

fn draw_storage_barchart(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    theme: &ThemeColor,
) {
    let st = &snapshot.storage;
    let img_mb = (st.images_total_size as f64 / MB).round() as u64;
    let cnt_mb = (st.containers_total_size as f64 / MB).round() as u64;
    let vol_mb = (st.volumes_total_size as f64 / MB).round() as u64;

    let bars = vec![
        Bar::default()
            .label("Images")
            .value(img_mb)
            .text_value(format_bytes(st.images_total_size))
            .style(Style::new().fg(theme.info)),
        Bar::default()
            .label("Cnts")
            .value(cnt_mb)
            .text_value(format_bytes(st.containers_total_size))
            .style(Style::new().fg(theme.status_running)),
        Bar::default()
            .label("Vols")
            .value(vol_mb)
            .text_value(format_bytes(st.volumes_total_size))
            .style(Style::new().fg(theme.warning)),
    ];

    let block = Block::bordered()
        .title(" Storage Breakdown (MB) ")
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let barchart = BarChart::default()
        .block(block)
        .bar_width(7)
        .bar_gap(2)
        .value_style(Style::new().fg(theme.text).add_modifier(Modifier::BOLD))
        .label_style(Style::new().fg(theme.text_muted))
        .data(BarGroup::default().bars(&bars));

    frame.render_widget(barchart, area);
}

fn draw_container_resource_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    cursor: usize,
    offset: usize,
    viewport: usize,
    theme: &ThemeColor,
) {
    let rows: Vec<Row> = snapshot
        .containers
        .iter()
        .skip(offset)
        .take(viewport)
        .map(|c| {
            let mem_str = format!(
                "{} ({:.1}%)",
                format_bytes(c.mem_usage),
                c.mem_percent
            );
            let disk_str = format!(
                "{} / {}",
                format_bytes(c.storage_rw),
                format_bytes(c.storage_rootfs)
            );
            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(c.state.clone()),
                Cell::from(format!("{:.2}%", c.cpu_percent)),
                Cell::from(mem_str),
                Cell::from(disk_str),
            ])
        })
        .collect();

    let header = Row::new([
        "CONTAINER",
        "STATE",
        "CPU %",
        "RAM USAGE",
        "STORAGE (RW / ROOTFS)",
    ])
    .style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(26),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Percentage(26),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(theme.border)
            .title(format!(
                " Container Resource Usage ({}) ",
                snapshot.containers.len()
            )),
    )
    .row_highlight_style(
        Style::new()
            .fg(theme.border)
            .bg(theme.text_selected)
            .add_modifier(Modifier::REVERSED),
    );

    let mut state = TableState::default();
    if !snapshot.containers.is_empty() {
        state.select(Some(cursor.saturating_sub(offset)));
    }
    frame.render_stateful_widget(table, area, &mut state);
}
