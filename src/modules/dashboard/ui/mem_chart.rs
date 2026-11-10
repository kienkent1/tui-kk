use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, BorderType, Chart, Dataset, Gauge, GraphType},
};

use crate::{
    modules::dashboard::dashboard_dto::{DashboardSnapshot, MetricHistory, format_bytes},
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    history: &MetricHistory,
    theme: &ThemeColor,
) {
    let used_str = format_bytes(snapshot.total_mem_usage);
    let limit_str = format_bytes(snapshot.total_mem_limit);

    let block = Block::bordered()
        .title(format!(
            " RAM Usage ({used_str} / {limit_str} · {:.1}%) ",
            snapshot.total_mem_percent
        ))
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 || inner.width < 6 {
        return;
    }

    let [gauge_area, chart_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);

    let ratio = (snapshot.total_mem_percent / 100.0).clamp(0.0, 1.0);
    let gauge_color = if snapshot.total_mem_percent >= 85.0 {
        theme.error
    } else if snapshot.total_mem_percent >= 60.0 {
        theme.warning
    } else {
        theme.info
    };

    let gauge = Gauge::default()
        .gauge_style(
            Style::new()
                .fg(gauge_color)
                .bg(theme.bg_panel)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(format!(
            "{used_str} / {limit_str} ({:.1}%)",
            snapshot.total_mem_percent
        ));
    frame.render_widget(gauge, gauge_area);

    let points = history.mem_dataset();
    let max_x = history.max_len().saturating_sub(1).max(1) as f64;

    let dataset = Dataset::default()
        .name("RAM %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::new().fg(gauge_color))
        .data(&points);

    let chart = Chart::new(vec![dataset])
        .style(Style::new().bg(theme.bg))
        .x_axis(
            Axis::default()
                .style(Style::new().fg(theme.text_muted))
                .bounds([0.0, max_x])
                .labels(vec![Span::raw("-60s"), Span::raw("now")]),
        )
        .y_axis(
            Axis::default()
                .style(Style::new().fg(theme.text_muted))
                .bounds([0.0, 100.0])
                .labels(vec![
                    Span::raw("0%"),
                    Span::raw("50%"),
                    Span::raw("100%"),
                ]),
        );

    frame.render_widget(chart, chart_area);
}

