use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, TableState},
};

use crate::{
    modules::images::{constants::RunField, image_dto::EnvVarEntry},
    tuikk_core::themes::ThemeColor,
};

#[derive(Debug, Clone, Default)]
pub struct RunViewAreas {
    pub name_area: Rect,
    pub ports_area: Rect,
    pub volumes_area: Rect,
    pub env_file_area: Rect,
    pub load_env_btn: Rect,
    pub import_img_env_btn: Rect,
    pub env_key_area: Rect,
    pub env_val_area: Rect,
    pub add_env_btn: Rect,
    pub clear_env_btn: Rect,
    pub env_list_area: Rect,
    pub run_btn: Rect,
    pub cancel_btn: Rect,
    pub env_viewport: usize,
}

pub struct RunViewProps<'a> {
    pub image_ref: &'a str,
    pub exposed_ports_hint: &'a [String],
    pub image_default_env_count: usize,
    pub focus: RunField,
    pub container_name: &'a str,
    pub ports_input: &'a str,
    pub volumes_input: &'a str,
    pub env_file_path: &'a str,
    pub env_key_input: &'a str,
    pub env_val_input: &'a str,
    pub env_vars: &'a [EnvVarEntry],
    pub env_cursor: usize,
    pub env_offset: usize,
    pub loading: bool,
    pub status_msg: Option<&'a str>,
    pub error_msg: Option<&'a str>,
}

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    props: &RunViewProps<'_>,
    areas: &mut RunViewAreas,
    theme: &ThemeColor,
) {
    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border_focus))
        .title(format!(
            " ▶ Run New Container — Image: {} ",
            props.image_ref
        ));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let [header_area, config_area, env_builder_area, env_list_area, footer_area] =
        Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .areas(inner);

    draw_header(frame, header_area, props, theme);
    draw_config_inputs(frame, config_area, props, areas, theme);
    draw_env_builder(frame, env_builder_area, props, areas, theme);
    draw_env_table(frame, env_list_area, props, areas, theme);
    draw_footer(frame, footer_area, props, areas, theme);
}

fn draw_header(frame: &mut Frame, area: Rect, props: &RunViewProps<'_>, theme: &ThemeColor) {
    let exposed = if props.exposed_ports_hint.is_empty() {
        "none".to_owned()
    } else {
        props.exposed_ports_hint.join(", ")
    };

    let mut line1_spans = vec![
        Span::styled(" Exposed Ports: ", Style::new().fg(theme.text_muted)),
        Span::styled(exposed, Style::new().fg(theme.info).bold()),
        Span::styled("   │   ", Style::new().fg(theme.border)),
        Span::styled("Image Default Env: ", Style::new().fg(theme.text_muted)),
        Span::styled(
            format!("{} var(s)", props.image_default_env_count),
            Style::new().fg(theme.accent).bold(),
        ),
    ];

    if props.loading {
        line1_spans.push(Span::styled(
            "   · Starting container…",
            Style::new().fg(theme.warning).bold(),
        ));
    } else if let Some(err) = props.error_msg {
        line1_spans.push(Span::styled(
            format!("   · Error: {err}"),
            Style::new().fg(theme.error).bold(),
        ));
    } else if let Some(msg) = props.status_msg {
        line1_spans.push(Span::styled(
            format!("   · {msg}"),
            Style::new().fg(theme.success).bold(),
        ));
    }

    let line2 = Line::from(vec![
        Span::styled(
            " Navigation: ",
            Style::new().fg(theme.text_muted),
        ),
        Span::styled("Tab / Shift+Tab", Style::new().fg(theme.keybind_key).bold()),
        Span::styled(" switch field · ", Style::new().fg(theme.keybind_desc)),
        Span::styled("Enter", Style::new().fg(theme.keybind_key).bold()),
        Span::styled(" load .env / add env / run · ", Style::new().fg(theme.keybind_desc)),
        Span::styled("Ctrl+R / F5", Style::new().fg(theme.keybind_key).bold()),
        Span::styled(" run container · ", Style::new().fg(theme.keybind_desc)),
        Span::styled("Esc", Style::new().fg(theme.keybind_key).bold()),
        Span::styled(" cancel", Style::new().fg(theme.keybind_desc)),
    ]);

    frame.render_widget(
        Paragraph::new(vec![Line::from(line1_spans), line2]),
        area,
    );
}

fn draw_config_inputs(
    frame: &mut Frame,
    area: Rect,
    props: &RunViewProps<'_>,
    areas: &mut RunViewAreas,
    theme: &ThemeColor,
) {
    let [row1, row2] = Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).areas(area);
    let [name_area, ports_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(row1);

    areas.name_area = name_area;
    areas.ports_area = ports_area;
    areas.volumes_area = row2;

    render_input_box(
        frame,
        name_area,
        " Container Name (optional) ",
        props.container_name,
        "e.g. my-app-container (leave blank for auto)",
        props.focus == RunField::ContainerName,
        theme,
    );

    render_input_box(
        frame,
        ports_area,
        " Port Mappings (host:container[/proto]) ",
        props.ports_input,
        "e.g. 8080:80, 5432:5432/tcp",
        props.focus == RunField::Ports,
        theme,
    );

    render_input_box(
        frame,
        row2,
        " Volume Mounts (host:container[:ro], comma-separated) ",
        props.volumes_input,
        "e.g. /host/path:/container/path, app_data:/var/lib/data:ro",
        props.focus == RunField::Volumes,
        theme,
    );
}

fn draw_env_builder(
    frame: &mut Frame,
    area: Rect,
    props: &RunViewProps<'_>,
    areas: &mut RunViewAreas,
    theme: &ThemeColor,
) {
    let [env_file_row, manual_env_row] =
        Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).areas(area);

    let [env_file_area, load_btn_area, import_btn_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(16),
        Constraint::Length(22),
    ])
    .areas(env_file_row);

    areas.env_file_area = env_file_area;
    areas.load_env_btn = load_btn_area;
    areas.import_img_env_btn = import_btn_area;

    render_input_box(
        frame,
        env_file_area,
        " Parse .env File Path (Enter to load) ",
        props.env_file_path,
        "e.g. .env or ~/project/.env",
        props.focus == RunField::EnvFilePath,
        theme,
    );

    render_action_button(
        frame,
        load_btn_area,
        "Load .env",
        props.focus == RunField::EnvFilePath,
        theme.info,
        theme,
    );

    render_action_button(
        frame,
        import_btn_area,
        "Import Image Env",
        false,
        theme.accent,
        theme,
    );

    let [key_area, val_area, add_btn_area, clear_btn_area] = Layout::horizontal([
        Constraint::Percentage(32),
        Constraint::Fill(1),
        Constraint::Length(16),
        Constraint::Length(14),
    ])
    .areas(manual_env_row);

    areas.env_key_area = key_area;
    areas.env_val_area = val_area;
    areas.add_env_btn = add_btn_area;
    areas.clear_env_btn = clear_btn_area;

    render_input_box(
        frame,
        key_area,
        " Env Key (or KEY=VALUE) ",
        props.env_key_input,
        "e.g. DB_HOST",
        props.focus == RunField::EnvKey,
        theme,
    );

    render_input_box(
        frame,
        val_area,
        " Env Value (Enter to add) ",
        props.env_val_input,
        "e.g. localhost",
        props.focus == RunField::EnvValue,
        theme,
    );

    render_action_button(
        frame,
        add_btn_area,
        "+ Add Env",
        props.focus == RunField::EnvValue,
        theme.status_running,
        theme,
    );

    render_action_button(
        frame,
        clear_btn_area,
        "Clear All",
        false,
        theme.status_stopped,
        theme,
    );
}

fn draw_env_table(
    frame: &mut Frame,
    area: Rect,
    props: &RunViewProps<'_>,
    areas: &mut RunViewAreas,
    theme: &ThemeColor,
) {
    areas.env_list_area = area;
    let viewport = area.height.saturating_sub(3).max(1) as usize;
    areas.env_viewport = viewport;

    let is_focused = props.focus == RunField::EnvList;
    let border_color = if is_focused {
        theme.border_focus
    } else {
        theme.border
    };

    let title = format!(
        " Environment Variables ({}) [↑/↓: Select · Enter/e: Edit · d/Del: Remove · c: Clear] ",
        props.env_vars.len()
    );

    let header = Row::new(["#", "KEY", "VALUE", "SOURCE"]).style(
        Style::new()
            .fg(theme.list_header_text)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = props
        .env_vars
        .iter()
        .enumerate()
        .skip(props.env_offset)
        .take(viewport)
        .map(|(idx, item)| {
            let src_color = match item.source.as_str() {
                "manual" => theme.status_running,
                "image" => theme.accent,
                _ => theme.info,
            };
            Row::new(vec![
                Cell::from((idx + 1).to_string()).style(Style::new().fg(theme.text_muted)),
                Cell::from(item.key.clone())
                    .style(Style::new().fg(theme.text_selected).add_modifier(Modifier::BOLD)),
                Cell::from(item.value.clone()).style(Style::new().fg(theme.input_text)),
                Cell::from(format!("[{}]", item.source)).style(Style::new().fg(src_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(32),
            Constraint::Fill(1),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .border_style(Style::new().fg(border_color))
            .title(title),
    )
    .row_highlight_style(
        Style::new()
            .fg(theme.border_focus)
            .bg(theme.bg_selected)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    if !props.env_vars.is_empty() {
        state.select(Some(props.env_cursor.saturating_sub(props.env_offset)));
    }
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_footer(
    frame: &mut Frame,
    area: Rect,
    props: &RunViewProps<'_>,
    areas: &mut RunViewAreas,
    theme: &ThemeColor,
) {
    let [_, run_btn_area, _, cancel_btn_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(28),
        Constraint::Length(3),
        Constraint::Length(18),
        Constraint::Fill(1),
    ])
    .areas(area);

    areas.run_btn = run_btn_area;
    areas.cancel_btn = cancel_btn_area;

    let run_focused = props.focus == RunField::RunButton;
    let run_border = if run_focused {
        theme.status_running
    } else {
        theme.border
    };
    let run_style = Style::new()
        .fg(theme.status_running)
        .add_modifier(Modifier::BOLD);

    frame.render_widget(
        Paragraph::new(" ▶ Run Container (Ctrl+R) ")
            .centered()
            .style(run_style)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(run_border)),
            ),
        run_btn_area,
    );

    frame.render_widget(
        Paragraph::new(" Esc Cancel ")
            .centered()
            .style(Style::new().fg(theme.status_stopped))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(theme.border)),
            ),
        cancel_btn_area,
    );
}

fn render_input_box(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    theme: &ThemeColor,
) {
    let border_color = if focused {
        theme.border_focus
    } else {
        theme.border
    };
    let bg_color = if focused {
        theme.input_bg_focus
    } else {
        theme.input_bg
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border_color))
        .style(Style::new().bg(bg_color))
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

    let (display_text, text_style) = if value.is_empty() && !focused {
        (placeholder, Style::new().fg(theme.input_placeholder))
    } else {
        (value, Style::new().fg(theme.input_text))
    };

    frame.render_widget(Paragraph::new(display_text).style(text_style), inner);

    if focused && inner.width > 0 && inner.height > 0 {
        let cursor_x = (inner.x + value.chars().count() as u16).min(inner.right().saturating_sub(1));
        frame.set_cursor_position(Position::new(cursor_x, inner.y));
    }
}

fn render_action_button(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    highlighted: bool,
    color: ratatui::style::Color,
    theme: &ThemeColor,
) {
    let border_color = if highlighted { color } else { theme.border };
    frame.render_widget(
        Paragraph::new(label)
            .centered()
            .style(Style::new().fg(color).add_modifier(Modifier::BOLD))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(border_color)),
            ),
        area,
    );
}

