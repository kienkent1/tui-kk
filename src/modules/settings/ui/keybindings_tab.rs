use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Cell, Row, Table, TableState},
};

use crate::{modules::settings::settings_dto::KeybindingRow, tuikk_core::themes::ThemeColor};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    rows: &[KeybindingRow],
    cursor: usize,
    offset: usize,
    theme: &ThemeColor,
) -> usize {
    let viewport = area.height.saturating_sub(3).max(1) as usize;

    let header = Row::new(["SCOPE", "KEY SEQUENCE", "COMMAND"]).style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let body: Vec<Row> = rows
        .iter()
        .skip(offset)
        .take(viewport)
        .map(|r| {
            Row::new(vec![
                Cell::from(format!("[{}]", r.scope)).style(Style::new().fg(theme.accent)),
                Cell::from(r.key_seq.clone())
                    .style(Style::new().fg(theme.keybind_key).add_modifier(Modifier::BOLD)),
                Cell::from(r.command.clone()).style(Style::new().fg(theme.text)),
            ])
        })
        .collect();

    let table = Table::new(
        body,
        [
            Constraint::Length(18),
            Constraint::Percentage(40),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(theme.border))
            .title(format!(
                " Active Keybindings ({}) [↑/↓: Scroll] ",
                rows.len()
            )),
    )
    .row_highlight_style(
        Style::new()
            .fg(theme.border_focus)
            .bg(theme.bg_selected)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    if !rows.is_empty() {
        state.select(Some(cursor.saturating_sub(offset)));
    }
    frame.render_stateful_widget(table, area, &mut state);
    viewport
}

