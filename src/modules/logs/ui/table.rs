use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Cell, Row, Table, TableState},
};

use crate::{
    modules::{
        dashboard::dashboard_dto::format_bytes,
        logs::log_dto::LogFileSummary,
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    rows: &[&LogFileSummary],
    offset: usize,
    viewport: usize,
    cursor: usize,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let len = rows.len();
    let body: Vec<Row> = rows
        .iter()
        .skip(offset)
        .take(viewport)
        .map(|f| {
            let err_style = if f.error_count > 0 {
                Style::new().fg(theme.error).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme.text_muted)
            };

            let warn_style = if f.warn_count > 0 {
                Style::new().fg(theme.warning).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme.text_muted)
            };

            let preview = f
                .last_error_preview
                .as_deref()
                .unwrap_or("(no errors/warnings)")
                .to_owned();

            Row::new(vec![
                Cell::from(f.file_name.clone()),
                Cell::from(format_bytes(f.size)),
                Cell::from(f.error_count.to_string()).style(err_style),
                Cell::from(f.warn_count.to_string()).style(warn_style),
                Cell::from(f.total_lines.to_string()),
                Cell::from(preview),
            ])
        })
        .collect();

    let header = Row::new([
        "LOG FILE",
        "SIZE",
        "ERRORS",
        "WARNINGS",
        "LINES",
        "LAST ERROR / WARNING",
    ])
    .style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let mut title = format!(" Log Files ({len}) ");
    if loading {
        title.push_str("· loading… ");
    }
    if let Some(e) = error {
        title.push_str(&format!("· error: {e} "));
    }

    let table = Table::new(
        body,
        [
            Constraint::Percentage(26), // LOG FILE
            Constraint::Length(12),     // SIZE
            Constraint::Length(10),     // ERRORS
            Constraint::Length(10),     // WARNINGS
            Constraint::Length(10),     // LINES
            Constraint::Fill(1),        // LAST ERROR / WARNING
        ],
    )
    .header(header)
    .block(Block::bordered().fg(theme.border).title(title))
    .row_highlight_style(
        Style::new()
            .fg(theme.border)
            .bg(theme.text_selected)
            .add_modifier(Modifier::REVERSED),
    );

    let mut state = TableState::default();
    if len > 0 {
        state.select(Some(cursor.saturating_sub(offset)));
    }
    frame.render_stateful_widget(table, area, &mut state);
}

