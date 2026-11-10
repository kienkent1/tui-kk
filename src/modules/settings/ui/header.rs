use std::path::Path;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs},
};

use crate::{modules::settings::constants::SettingsTab, tuikk_core::themes::ThemeColor};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    config_path: &Path,
    active_tab: SettingsTab,
    dirty: bool,
    status_msg: Option<&str>,
    error_msg: Option<&str>,
    theme: &ThemeColor,
) {
    let [info_area, tab_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).areas(area);

    let mut status_spans = vec![
        Span::styled(" Config File: ", Style::new().fg(theme.text_muted)),
        Span::styled(
            config_path.display().to_string(),
            Style::new().fg(theme.info).add_modifier(Modifier::BOLD),
        ),
    ];

    if dirty {
        status_spans.push(Span::styled(
            "   · [Unsaved Changes]",
            Style::new().fg(theme.warning).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(err) = error_msg {
        status_spans.push(Span::styled(
            format!("   · Error: {err}"),
            Style::new().fg(theme.error).add_modifier(Modifier::BOLD),
        ));
    } else if let Some(msg) = status_msg {
        status_spans.push(Span::styled(
            format!("   · {msg}"),
            Style::new().fg(theme.success).add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(status_spans)).block(
            Block::bordered()
                .border_style(Style::new().fg(theme.border))
                .title(" Settings & Configuration "),
        ),
        info_area,
    );

    let titles: Vec<Line> = SettingsTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let idx = SettingsTab::ALL
        .iter()
        .position(|&t| t == active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(idx)
        .block(
            Block::bordered()
                .border_style(Style::new().fg(theme.border))
                .title(" Section [1-3 / h-l] "),
        )
        .style(Style::new().fg(theme.tab_inactive_text))
        .highlight_style(
            Style::new()
                .fg(theme.border_focus)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, tab_area);
}

