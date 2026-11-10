use std::path::Path;

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
    log_dir: &Path,
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

    let dir_str = log_dir.display();
    let sort_badge = match sort {
        Some(s) if !s.is_empty() => format!(" · sort: {s} "),
        _ => String::from(" "),
    };

    let block = Block::bordered()
        .title(format!(
            " Search Logs (/) · dir: {dir_str}{sort_badge}[d: Delete | c: Clear | p: Delete All] "
        ))
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

