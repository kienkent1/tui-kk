use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    modules::settings::{
        constants::DockerField,
        settings_dto::SettingsDraft,
        ui::general_tab::{render_numeric_field, render_text_field},
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    draft: &SettingsDraft,
    focus: DockerField,
    editing_text: bool,
    theme: &ThemeColor,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(" Docker Engine / CLI Connection [↑/↓: Select Field · ←/→/Space: Cycle · Enter: Edit · t: Test Ping] ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(inner);

    let [timeout_area, version_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(rows[2]);

    // Connection type selector
    let conn_focused = focus == DockerField::ConnectionType;
    let border = if conn_focused {
        theme.border_focus
    } else {
        theme.border
    };
    let current = draft.conn_type_label();
    let mut spans = vec![Span::raw(" ")];
    for (i, opt) in ["socket", "local", "http"].iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::new().fg(theme.border)));
        }
        if *opt == current {
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
                    " Default Connection Type (socket / local / http) ",
                    Style::new().fg(if conn_focused {
                        theme.border_focus
                    } else {
                        theme.text_muted
                    }),
                )),
        ),
        rows[0],
    );

    render_text_field(
        frame,
        rows[1],
        " Socket Path or HTTP/TCP Address (Enter to edit, e.g. /var/run/docker.sock or tcp://127.0.0.1:2375) ",
        &draft.addr_or_path,
        "default (/var/run/docker.sock or system default)",
        focus == DockerField::AddrOrPath,
        focus == DockerField::AddrOrPath && editing_text,
        theme,
    );

    render_numeric_field(
        frame,
        timeout_area,
        " Timeout in Seconds (← / → / - / +) ",
        &format!("{} s", draft.timeout),
        focus == DockerField::Timeout,
        theme,
    );

    render_text_field(
        frame,
        version_area,
        " Client API Version (optional, Enter to edit) ",
        &draft.client_version,
        "default (negotiate or bollard default)",
        focus == DockerField::ClientVersion,
        focus == DockerField::ClientVersion && editing_text,
        theme,
    );
}

