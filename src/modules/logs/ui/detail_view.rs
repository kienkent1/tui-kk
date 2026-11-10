use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Tabs},
};

use crate::{
    modules::logs::{
        constants::LogDetailTab,
        log_dto::{LogEntry, LogLevel},
        log_service::LogService,
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    file_name: &str,
    entries: &[LogEntry],
    active_tab: LogDetailTab,
    scroll: u16,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let [header_area, tabs_area, body_area] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .areas(area);

    draw_header(frame, header_area, file_name, entries, loading, error, theme);
    draw_tabs(frame, tabs_area, active_tab, theme);
    draw_body(frame, body_area, entries, active_tab, scroll, theme);
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    file_name: &str,
    entries: &[LogEntry],
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let err_cnt = entries.iter().filter(|e| e.level.is_error()).count();
    let warn_cnt = entries
        .iter()
        .filter(|e| e.level == LogLevel::Warn)
        .count();

    let mut title = format!(" Log File: {file_name} ");
    if loading {
        title.push_str("· loading… ");
    }
    if let Some(e) = error {
        title.push_str(&format!("· error: {e} "));
    }

    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    let lines = vec![
        Line::from(vec![
            Span::styled(" Errors: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                err_cnt.to_string(),
                Style::new().fg(theme.error).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Warnings: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                warn_cnt.to_string(),
                Style::new().fg(theme.warning).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Total Lines: ", Style::new().fg(theme.text_muted)),
            Span::styled(entries.len().to_string(), Style::new().fg(theme.text)),
        ]),
        Line::from(vec![Span::styled(
            " [Esc: Back | 1-4: Filter Level | d: Delete File | c: Clear File | j/k: Scroll]",
            Style::new().fg(theme.text_muted),
        )]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, active_tab: LogDetailTab, theme: &ThemeColor) {
    let titles: Vec<Line> = LogDetailTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let selected = LogDetailTab::ALL
        .iter()
        .position(|&t| t == active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(theme.border)),
        )
        .select(selected)
        .style(Style::new().fg(theme.tab_inactive_text))
        .highlight_style(
            Style::new()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, area);
}

fn draw_body(
    frame: &mut Frame,
    area: Rect,
    entries: &[LogEntry],
    active_tab: LogDetailTab,
    scroll: u16,
    theme: &ThemeColor,
) {
    let filtered = LogService::filter_entries_by_tab(entries, active_tab);
    let block = Block::bordered()
        .title(format!(" {} ({}) ", active_tab.title(), filtered.len()))
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    if filtered.is_empty() {
        frame.render_widget(
            Paragraph::new(" (no matching log entries)")
                .style(Style::new().fg(theme.text_muted))
                .block(block),
            area,
        );
        return;
    }

    let lines: Vec<Line> = filtered
        .into_iter()
        .map(|e| {
            if active_tab == LogDetailTab::Raw || e.level == LogLevel::Unknown {
                return Line::from(Span::styled(
                    format!("{:>4} │ {}", e.line_number, e.raw),
                    Style::new().fg(theme.text),
                ));
            }

            let level_style = match e.level {
                LogLevel::Error => Style::new().fg(theme.error).add_modifier(Modifier::BOLD),
                LogLevel::Warn => Style::new().fg(theme.warning).add_modifier(Modifier::BOLD),
                LogLevel::Info => Style::new().fg(theme.info),
                LogLevel::Debug | LogLevel::Trace | LogLevel::Unknown => {
                    Style::new().fg(theme.text_muted)
                }
            };

            Line::from(vec![
                Span::styled(
                    format!("{:>4} │ ", e.line_number),
                    Style::new().fg(theme.text_muted),
                ),
                Span::styled(
                    format!("{:<5} ", e.level.as_str()),
                    level_style,
                ),
                Span::styled(
                    format!("{} ", e.timestamp),
                    Style::new().fg(theme.text_muted),
                ),
                Span::styled(
                    if e.target.is_empty() {
                        String::new()
                    } else {
                        format!("[{}] ", e.target)
                    },
                    Style::new().fg(theme.accent),
                ),
                Span::styled(e.message.clone(), Style::new().fg(theme.text)),
            ])
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll, 0)),
        area,
    );
}

