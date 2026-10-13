use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::{
    shared::base::base_widget::BaseWidget,
    tuikk_core::{config::AppConfig, themes::ThemeColor},
};

#[derive(Debug, Clone)]
pub enum ButtonEvent {
    Focused,
    Hovered,
    Clicked,
}
#[derive(Debug, Clone)]
pub struct Button {
    label: String,
    hovered: bool,
    pressed: bool,
    bg: Color,
    border_color: Color,
    hover_color: Color,
    focus_color: Color,
    text_color: Color,

    button_style: Style,
    hover_style: Style,
    press_style: Style,

    border_type: BorderType,
}

impl Button {
    pub fn new(label: String, app_config: &Option<ThemeColor>) -> Self {
        let color = match app_config {
            Some(c) => *c,
            _ => AppConfig::global().theme_color,
        };
        Self {
            label: label,
            hovered: false,
            pressed: false,
            bg: color.bg,
            border_color: color.border,
            hover_color: color.border_focus,
            focus_color: color.border_focus,
            text_color: color.text,
            button_style: Style::new(),
            hover_style: Style::new(),
            press_style: Style::new(),
            border_type: BorderType::Rounded,
        }
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    pub fn hover(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }
    pub fn focus(mut self, color: Color) -> Self {
        self.focus_color = color;
        self
    }
    pub fn border(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
    pub fn button_style(mut self, style: Style) -> Self {
        self.button_style = style;
        self
    }
    pub fn hover_style(mut self, style: Style) -> Self {
        self.hover_style = style;
        self
    }
    pub fn press_style(mut self, style: Style) -> Self {
        self.press_style = style;
        self
    }
}

impl BaseWidget for Button {
    type Event = ButtonEvent;
    fn handle_key(&mut self, key: KeyEvent) -> Option<ButtonEvent> {
        None
    }
    fn handle_mouse(&mut self, area: Rect, mouse: MouseEvent) -> Option<ButtonEvent> {
        let inside = area.contains(Position::new(mouse.column, mouse.row));
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hovered = inside;
                Some(ButtonEvent::Hovered)
            }
            // Press: start the "active" state. No click yet.
            MouseEventKind::Down(MouseButton::Left) => {
                if inside {
                    self.pressed = true;
                    return Some(ButtonEvent::Focused);
                }
                None
            }
            // Drag: the cursor may leave the button while the button is held.
            MouseEventKind::Drag(MouseButton::Left) => {
                self.hovered = inside;
                Some(ButtonEvent::Hovered)
            }
            // Release: click only if the press started here and ended here.
            MouseEventKind::Up(MouseButton::Left) => {
                let was_pressed = std::mem::take(&mut self.pressed);
                if was_pressed && inside {
                    Some(ButtonEvent::Clicked)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let style = if self.pressed {
            self.press_style
        } else if self.hovered {
            self.hover_style
        } else {
            self.button_style
        };

        Paragraph::new(self.label.as_str())
            .style(style)
            .block(Block::bordered().border_type(self.border_type))
            .render(area, buf);
    }
}
