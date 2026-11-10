use bollard::models::ImageInspect;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Tabs},
};

use crate::{
    modules::images::{
        constants::ImageDetailTab,
        image_dto::{format_image_size, short_image_id},
    },
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    image: &ImageInspect,
    active_tab: ImageDetailTab,
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

    draw_header(frame, header_area, image, loading, error, theme);
    draw_tabs(frame, tabs_area, active_tab, theme);
    draw_body(frame, body_area, image, active_tab, scroll, theme);
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    image: &ImageInspect,
    loading: bool,
    error: Option<&str>,
    theme: &ThemeColor,
) {
    let short_id = image.id.as_deref().map(short_image_id).unwrap_or("-");
    let primary_tag = image
        .repo_tags
        .as_ref()
        .and_then(|v| v.first())
        .map(String::as_str)
        .unwrap_or("<none>");
    let size = format_image_size(image.size.unwrap_or(0));
    let os_arch = format!(
        "{}/{}",
        image.os.as_deref().unwrap_or("-"),
        image.architecture.as_deref().unwrap_or("-")
    );

    let mut title = format!(" Image: {primary_tag} ({short_id}) ");
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
            Span::styled(" Size: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                size,
                Style::new().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Platform: ", Style::new().fg(theme.text_muted)),
            Span::styled(os_arch, Style::new().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" Created: ", Style::new().fg(theme.text_muted)),
            Span::styled(
                image
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

fn draw_tabs(frame: &mut Frame, area: Rect, active_tab: ImageDetailTab, theme: &ThemeColor) {
    let titles: Vec<Line> = ImageDetailTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let selected = ImageDetailTab::ALL
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
    image: &ImageInspect,
    active_tab: ImageDetailTab,
    scroll: u16,
    theme: &ThemeColor,
) {
    let content = match active_tab {
        ImageDetailTab::Overview => build_overview_text(image),
        ImageDetailTab::Layers => build_layers_text(image),
        ImageDetailTab::Config => build_config_text(image),
        ImageDetailTab::RawJson => json5::to_string(image)
            .unwrap_or_else(|err| format!("Failed to serialize image: {err}")),
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

fn build_overview_text(img: &ImageInspect) -> String {
    let mut out = Vec::new();
    out.push(format!(
        "ID:           {}",
        img.id.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "RepoTags:     {}",
        img.repo_tags
            .as_ref()
            .map(|v| v.join(", "))
            .unwrap_or_else(|| "-".to_owned())
    ));
    out.push(format!(
        "RepoDigests:  {}",
        img.repo_digests
            .as_ref()
            .map(|v| v.join(", "))
            .unwrap_or_else(|| "-".to_owned())
    ));
    out.push(format!(
        "Author:       {}",
        img.author.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "Architecture: {}",
        img.architecture.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "OS:           {}",
        img.os.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "Size:         {}",
        format_image_size(img.size.unwrap_or(0))
    ));
    out.join("\n")
}

fn build_layers_text(img: &ImageInspect) -> String {
    let Some(rootfs) = &img.root_fs else {
        return "(no rootfs information)".to_owned();
    };
    let mut out = Vec::new();
    out.push(format!("Type: {}", rootfs.typ));
    out.push(String::new());
    out.push("── Layers ──".to_owned());
    if let Some(layers) = &rootfs.layers {
        for (i, layer) in layers.iter().enumerate() {
            out.push(format!("  {:>2}. {layer}", i + 1));
        }
    } else {
        out.push("  (no layers)".to_owned());
    }
    out.join("\n")
}

fn build_config_text(img: &ImageInspect) -> String {
    let Some(cfg) = &img.config else {
        return "(no config)".to_owned();
    };
    let mut out = Vec::new();
    out.push(format!(
        "WorkingDir: {}",
        cfg.working_dir.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "User:       {}",
        cfg.user.as_deref().unwrap_or("-")
    ));
    out.push(format!(
        "Entrypoint: {}",
        cfg.entrypoint
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_else(|| "-".to_owned())
    ));
    out.push(format!(
        "Cmd:        {}",
        cfg.cmd
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_else(|| "-".to_owned())
    ));
    out.push(String::new());
    out.push("── Environment ──".to_owned());
    if let Some(envs) = &cfg.env {
        for e in envs {
            out.push(format!("  {e}"));
        }
    } else {
        out.push("  (none)".to_owned());
    }
    out.join("\n")
}
