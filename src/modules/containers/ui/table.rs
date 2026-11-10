use bollard::plugin::ContainerSummary;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Cell, Row, Table, TableState},
};

use crate::{
    modules::containers::container_dto::{
        container_display_name, format_ports, short_container_id,
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    rows: &[&ContainerSummary],
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
        .map(|c| {
            let short_id = short_container_id(c);
            let name = container_display_name(c);
            let ports = format_ports(c.ports.as_deref());

            Row::new(vec![
                Cell::from(short_id.to_owned()),
                Cell::from(name.to_owned()),
                Cell::from(c.image.clone().unwrap_or_default()),
                Cell::from(ports),
                Cell::from(c.state.map(|s| s.to_string()).unwrap_or_default()),
                Cell::from(c.status.clone().unwrap_or_default()),
            ])
        })
        .collect();

    let header = Row::new(["ID", "NAME", "IMAGE", "PORT", "STATE", "STATUS"]).style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let mut title = format!(" Containers ({len}) ");
    if loading {
        title.push_str("· loading… ");
    }
    if let Some(e) = error {
        title.push_str(&format!("· error: {e} "));
    }

    let table = Table::new(
        body,
        [
            Constraint::Length(13),     // ID   (12 chars + 1 padding)
            Constraint::Percentage(20), // NAME
            Constraint::Percentage(25), // IMAGE
            Constraint::Percentage(20), // PORTS
            Constraint::Length(10),     // STATE
            Constraint::Fill(1),        // STATUS
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
