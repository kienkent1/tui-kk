use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Paragraph},
};

use crate::tuikk_core::themes::ThemeColor;

#[derive(Debug, Clone, Default)]
pub struct FooterButtonAreas {
    pub save_btn: Rect,
    pub reload_btn: Rect,
    pub defaults_btn: Rect,
    pub ping_btn: Rect,
}

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    btns: &mut FooterButtonAreas,
    theme: &ThemeColor,
) {
    let [_, save_rect, _, reload_rect, _, defaults_rect, _, ping_rect, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(22),
        Constraint::Length(2),
        Constraint::Length(22),
        Constraint::Length(2),
        Constraint::Length(24),
        Constraint::Length(2),
        Constraint::Length(22),
        Constraint::Fill(1),
    ])
    .areas(area);

    btns.save_btn = save_rect;
    btns.reload_btn = reload_rect;
    btns.defaults_btn = defaults_rect;
    btns.ping_btn = ping_rect;

    frame.render_widget(
        Paragraph::new(" Save Config (w) ")
            .centered()
            .style(
                Style::new()
                    .fg(theme.status_running)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(theme.border)),
            ),
        save_rect,
    );

    frame.render_widget(
        Paragraph::new(" Reload (F5) ")
            .centered()
            .style(Style::new().fg(theme.info).add_modifier(Modifier::BOLD))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(theme.border)),
            ),
        reload_rect,
    );

    frame.render_widget(
        Paragraph::new(" Defaults (Ctrl+D) ")
            .centered()
            .style(Style::new().fg(theme.warning).add_modifier(Modifier::BOLD))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(theme.border)),
            ),
        defaults_rect,
    );

    frame.render_widget(
        Paragraph::new(" Test Ping (t) ")
            .centered()
            .style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(theme.border)),
            ),
        ping_rect,
    );
}
