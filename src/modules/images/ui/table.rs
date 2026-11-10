use bollard::models::ImageSummary;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Cell, Row, Table, TableState},
};

use crate::{
    modules::images::image_dto::{format_image_size, parse_repo_tag, short_image_id},
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    rows: &[&ImageSummary],
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
        .map(|img| {
            let short_id = short_image_id(&img.id).to_owned();
            let (repo, tag) = parse_repo_tag(img);
            let size = format_image_size(img.size);
            let containers = if img.containers < 0 {
                "-".to_owned()
            } else {
                img.containers.to_string()
            };
            let created = img.created.to_string();

            Row::new(vec![
                Cell::from(short_id),
                Cell::from(repo),
                Cell::from(tag),
                Cell::from(size),
                Cell::from(containers),
                Cell::from(created),
            ])
        })
        .collect();

    let header = Row::new([
        "ID",
        "REPOSITORY",
        "TAG",
        "SIZE",
        "CONTAINERS",
        "CREATED (EPOCH)",
    ])
    .style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let mut title = format!(" Images ({len}) ");
    if loading {
        title.push_str("· loading… ");
    }
    if let Some(e) = error {
        title.push_str(&format!("· error: {e} "));
    }

    let table = Table::new(
        body,
        [
            Constraint::Length(13),     // ID
            Constraint::Percentage(35), // REPOSITORY
            Constraint::Percentage(18), // TAG
            Constraint::Length(12),     // SIZE
            Constraint::Length(12),     // CONTAINERS
            Constraint::Fill(1),        // CREATED
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
