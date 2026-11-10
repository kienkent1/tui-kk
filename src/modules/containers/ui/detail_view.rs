use bollard::plugin::ContainerInspectResponse;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Tabs},
};

use crate::{modules::containers::constants::ContainerDetailTab, tuikk_core::themes::ThemeColor};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    container: &ContainerInspectResponse,
    active_tab: ContainerDetailTab,
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

    draw_header(frame, header_area, container, loading, error, theme);
    draw_tabs(frame, tabs_area, active_tab, theme);
    draw_body(frame, body_area, container, active_tab, scroll, theme);
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    container: &ContainerInspectResponse,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let name = container
        .name
        .as_deref()
        .map(|s| s.trim_start_matches('/'))
        .unwrap_or("Container");
    let short_id = container
        .id
        .as_deref()
        .map(|s| &s[..s.len().min(12)])
        .unwrap_or("-");
    let status = container
        .state
        .as_ref()
        .and_then(|st| st.status.map(|s| s.to_string()))
        .unwrap_or_else(|| "-".to_owned());
    let image = container
        .config
        .as_ref()
        .and_then(|cfg| cfg.image.as_deref())
        .or(container.image.as_deref())
        .unwrap_or("-");

    let mut title = format!(" Container: {name} ({short_id}) ");
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
            Span::styled(" Image:  ", Style::new().fg(theme.text_muted)),
            Span::styled(
                image,
                Style::new().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    State: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                status,
                Style::new()
                    .fg(theme.status_running)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Created: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                container
                    .created
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "-".to_owned()),
                Style::new().fg(theme.text),
            ),
            Span::styled(
                "    [Esc: Back | 1-4: Tab | j/k: Scroll]",
                Style::new().fg(theme.text_muted),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, active_tab: ContainerDetailTab, theme: &ThemeColor) {
    let titles: Vec<Line> = ContainerDetailTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let selected = ContainerDetailTab::ALL
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
    container: &ContainerInspectResponse,
    active_tab: ContainerDetailTab,
    scroll: u16,
    theme: &ThemeColor,
) {
    let content = match active_tab {
        ContainerDetailTab::Overview => build_overview_text(container),
        ContainerDetailTab::NetworkMounts => build_network_mounts_text(container),
        ContainerDetailTab::Env => build_env_text(container),
        ContainerDetailTab::RawJson => json5::to_string(container)
            .unwrap_or_else(|err| format!("Failed to serialize container: {err}")),
    };

    let block = Block::bordered()
        .title(format!(" {} ", active_tab.title()))
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border));

    frame.render_widget(
        Paragraph::new(content)
            .style(Style::new().fg(theme.text))
            .block(block)
            .scroll((scroll, 0)),
        area,
    );
}

fn build_overview_text(c: &ContainerInspectResponse) -> String {
    let mut out = Vec::new();
    out.push(format!("ID:          {}", c.id.as_deref().unwrap_or("-")));
    out.push(format!(
        "Name:        {}",
        c.name
            .as_deref()
            .map(|s| s.trim_start_matches('/'))
            .unwrap_or("-")
    ));
    out.push(format!(
        "Platform:    {}",
        c.platform.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "Driver:      {}",
        c.driver.as_deref().unwrap_or("-")
    ));
    out.push(format!("RestartCnt:  {}", c.restart_count.unwrap_or(0)));

    if let Some(st) = &c.state {
        out.push(String::new());
        out.push("── State ──".to_owned());
        out.push(format!(
            "  Status:     {}",
            st.status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_owned())
        ));
        out.push(format!("  Running:    {}", st.running.unwrap_or(false)));
        out.push(format!("  Pid:        {}", st.pid.unwrap_or(0)));
        out.push(format!("  ExitCode:   {}", st.exit_code.unwrap_or(0)));
        out.push(format!(
            "  StartedAt:  {}",
            st.started_at.as_deref().unwrap_or("-")
        ));
        out.push(format!(
            "  FinishedAt: {}",
            st.finished_at.as_deref().unwrap_or("-")
        ));
    }

    if let Some(cfg) = &c.config {
        out.push(String::new());
        out.push("── Config ──".to_owned());
        out.push(format!(
            "  Hostname:   {}",
            cfg.hostname.as_deref().unwrap_or("-")
        ));
        out.push(format!(
            "  WorkingDir: {}",
            cfg.working_dir.as_deref().unwrap_or("-")
        ));
        let entrypoint = cfg
            .entrypoint
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_else(|| "-".to_owned());
        let cmd = cfg
            .cmd
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_else(|| "-".to_owned());
        out.push(format!("  Entrypoint: {entrypoint}"));
        out.push(format!("  Cmd:        {cmd}"));
    }

    out.join("\n")
}

fn build_network_mounts_text(c: &ContainerInspectResponse) -> String {
    let mut out = Vec::new();
    out.push("── Networks ──".to_owned());
    if let Some(net) = &c.network_settings {
        if let Some(networks) = &net.networks {
            for (name, ep) in networks {
                out.push(format!(
                    "  • {name}: IP={} Gateway={} MAC={}",
                    ep.ip_address.as_deref().unwrap_or("-"),
                    ep.gateway.as_deref().unwrap_or("-"),
                    ep.mac_address.as_deref().unwrap_or("-"),
                ));
            }
        }
    }

    out.push(String::new());
    out.push("── Mounts ──".to_owned());
    if let Some(mounts) = &c.mounts {
        if mounts.is_empty() {
            out.push("  (no mounts)".to_owned());
        }
        for m in mounts {
            out.push(format!(
                "  • {} -> {} (rw: {})",
                m.source.as_deref().unwrap_or("-"),
                m.destination.as_deref().unwrap_or("-"),
                m.rw.unwrap_or(false)
            ));
        }
    } else {
        out.push("  (no mounts)".to_owned());
    }

    out.join("\n")
}

fn build_env_text(c: &ContainerInspectResponse) -> String {
    let Some(cfg) = &c.config else {
        return "(no environment variables)".to_owned();
    };
    let Some(envs) = &cfg.env else {
        return "(no environment variables)".to_owned();
    };
    if envs.is_empty() {
        return "(no environment variables)".to_owned();
    }
    envs.join("\n")
}
