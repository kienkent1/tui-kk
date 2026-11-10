use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::tuikk_core::themes::ThemeColor;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    input_mode: bool,
    draft: &str,
    search: Option<&str>,
    sort: Option<&str>,
    is_local: bool,
    theme: &ThemeColor,
) {
    let (text, border) = if input_mode {
        (draft.to_owned(), Style::new().fg(theme.border_focus))
    } else {
        (
            search.unwrap_or_default().to_owned(),
            Style::new().fg(theme.border),
        )
    };

    let mode = if is_local { "local" } else { "remote" };
    let sort_badge = match sort {
        Some(s) if !s.is_empty() => format!(" · sort: {s} "),
        _ => String::from(" "),
    };

    let block = Block::bordered()
        .title(format!(" Search Images (/) · {mode}{sort_badge}"))
        .border_style(border);
    let inner = block.inner(area);

    let text_color = Style::new().fg(theme.input_cursor);
    frame.render_widget(Paragraph::new(text).style(text_color).block(block), area);

    if input_mode {
        let w = Line::from(draft).width() as u16;
        let x = (inner.x + w).min(inner.right().saturating_sub(1));
        frame.set_cursor_position((x, inner.y));
    }
}
