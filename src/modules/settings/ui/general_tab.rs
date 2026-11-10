use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    modules::settings::{constants::GeneralField, settings_dto::SettingsDraft},
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    draft: &SettingsDraft,
    focus: GeneralField,
    editing_text: bool,
    theme: &ThemeColor,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(" General, Appearance & Logging [↑/↓: Select Field · ←/→/Space: Cycle · Enter: Edit] ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(inner);

    let [theme_area, icon_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(rows[0]);
    let [query_area, level_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(rows[1]);
    let [fps_area, tick_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(rows[3]);

    render_selector(
        frame,
        theme_area,
        " Theme (← / → / Space) ",
        &draft.theme,
        &["dark", "light"],
        focus == GeneralField::Theme,
        theme,
    );

    render_selector(
        frame,
        icon_area,
        " Icon Set (← / → / Space) ",
        draft.icon_set_label(),
        &["ascii", "unicode", "nerd"],
        focus == GeneralField::IconSet,
        theme,
    );

    render_selector(
        frame,
        query_area,
        " Query Mode (← / → / Space) ",
        draft.query_mode_label(),
        &["local", "remote"],
        focus == GeneralField::QueryMode,
        theme,
    );

    render_selector(
        frame,
        level_area,
        " Log Level (← / → / Space) ",
        &draft.log_level,
        &["error", "warn", "info", "debug", "trace"],
        focus == GeneralField::LogLevel,
        theme,
    );

    render_text_field(
        frame,
        rows[2],
        " Log Directory (Enter to edit text, Esc/Enter to finish) ",
        &draft.log_dir,
        "default (~/.local/share/tuikk/logs)",
        focus == GeneralField::LogDir,
        focus == GeneralField::LogDir && editing_text,
        theme,
    );

    render_numeric_field(
        frame,
        fps_area,
        " UI Frame Rate (FPS) (← / → / - / +) ",
        &format!("{:.1} FPS", draft.frame_rate),
        focus == GeneralField::FrameRate,
        theme,
    );

    render_numeric_field(
        frame,
        tick_area,
        " UI Tick Rate (Hz) (← / → / - / +) ",
        &format!("{:.1} Hz", draft.tick_rate),
        focus == GeneralField::TickRate,
        theme,
    );
}

fn render_selector(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    current: &str,
    options: &[&str],
    focused: bool,
    theme: &ThemeColor,
) {
    let border = if focused {
        theme.border_focus
    } else {
        theme.border
    };
    let mut spans = vec![Span::raw(" ")];
    for (i, &opt) in options.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::new().fg(theme.border)));
        }
        if opt.eq_ignore_ascii_case(current) {
            spans.push(Span::styled(
                format!("● {opt}"),
                Style::new()
                    .fg(theme.status_running)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("○ {opt}"),
                Style::new().fg(theme.text_muted),
            ));
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(border))
                .title(Span::styled(
                    title,
                    Style::new().fg(if focused {
                        theme.border_focus
                    } else {
                        theme.text_muted
                    }),
                )),
        ),
        area,
    );
}

pub fn render_text_field(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    editing: bool,
    theme: &ThemeColor,
) {
    let border = if editing {
        theme.status_running
    } else if focused {
        theme.border_focus
    } else {
        theme.border
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .title(Span::styled(
            title,
            Style::new().fg(if focused {
                theme.border_focus
            } else {
                theme.text_muted
            }),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (txt, style) = if value.is_empty() && !editing {
        (placeholder, Style::new().fg(theme.input_placeholder))
    } else {
        (value, Style::new().fg(theme.input_text))
    };

    frame.render_widget(Paragraph::new(txt).style(style), inner);

    if editing && inner.width > 0 && inner.height > 0 {
        let cx = (inner.x + value.chars().count() as u16).min(inner.right().saturating_sub(1));
        frame.set_cursor_position(Position::new(cx, inner.y));
    }
}

pub fn render_numeric_field(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value_label: &str,
    focused: bool,
    theme: &ThemeColor,
) {
    let border = if focused {
        theme.border_focus
    } else {
        theme.border
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ◀ ", Style::new().fg(theme.text_muted)),
            Span::styled(
                value_label,
                Style::new()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ▶ ", Style::new().fg(theme.text_muted)),
        ]))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(border))
                .title(Span::styled(
                    title,
                    Style::new().fg(if focused {
                        theme.border_focus
                    } else {
                        theme.text_muted
                    }),
                )),
        ),
        area,
    );
}

