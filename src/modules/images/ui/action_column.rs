use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{
    modules::images::{
        actions::ImageButtonAction,
        image_dto::{ImageRowButton, ImageRowTarget},
    },
    shared::{base::icons::Icons, helpers::get_color_helpers::BtnPalette},
    tuikk_core::themes::ThemeColor,
};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    total_len: usize,
    offset: usize,
    viewport: usize,
    hovered: Option<ImageRowTarget>,
    pressed: Option<ImageRowTarget>,
    row_buttons: &mut Vec<ImageRowButton>,
    theme: &ThemeColor,
) {
    let block = Block::bordered().border_style(Style::new().fg(theme.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    frame.render_widget(
        Paragraph::new("Action").centered().style(
            Style::new()
                .fg(theme.list_header_text)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );

    row_buttons.clear();
    let body_top = inner.y + 1;
    let run_icon = Icons::Start.glyph();
    let remove_icon = Icons::Remove.glyph();

    for (i, idx) in (offset..total_len).take(viewport).enumerate() {
        let row_rect = Rect::new(inner.x, body_top + i as u16, inner.width, 1);
        let [run_cell, remove_cell] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                .flex(Flex::SpaceEvenly)
                .areas(row_rect);

        let run_target = ImageRowTarget {
            row: idx,
            action: ImageButtonAction::Run,
        };
        let remove_target = ImageRowTarget {
            row: idx,
            action: ImageButtonAction::Remove,
        };

        let run_rect = button_rect(run_cell, run_icon);
        let remove_rect = button_rect(remove_cell, remove_icon);

        frame.render_widget(
            Paragraph::new(format!(" {run_icon} ")).style(button_style(
                run_target,
                hovered,
                pressed,
                theme.btn_start(),
            )),
            run_rect,
        );
        frame.render_widget(
            Paragraph::new(format!(" {remove_icon} ")).style(button_style(
                remove_target,
                hovered,
                pressed,
                theme.btn_remove(),
            )),
            remove_rect,
        );

        row_buttons.push(ImageRowButton {
            target: run_target,
            rect: run_rect,
        });
        row_buttons.push(ImageRowButton {
            target: remove_target,
            rect: remove_rect,
        });
    }
}

fn button_rect(cell: Rect, label: &str) -> Rect {
    let w = (Line::from(label).width() as u16 + 2).min(cell.width);
    let [rect] = Layout::horizontal([Constraint::Length(w)])
        .flex(Flex::Center)
        .areas(cell);
    rect
}

fn button_style(
    target: ImageRowTarget,
    hovered: Option<ImageRowTarget>,
    pressed: Option<ImageRowTarget>,
    p: BtnPalette,
) -> Style {
    if pressed == Some(target) {
        Style::new().fg(p.fg_press).add_modifier(Modifier::BOLD)
    } else if hovered == Some(target) {
        Style::new().fg(p.fg_hover).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(p.fg)
    }
}
