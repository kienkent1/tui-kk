use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};
use std::sync::LazyLock;

use crate::{
    shared::base::base_widget::BaseWidget,
    tuikk_core::{
        route::{Pages, Route},
        themes::ThemeColor,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideBarEvent {
    NavigatePage(Route),
}

pub static PAGES: LazyLock<Vec<Pages>> = LazyLock::new(|| {
    vec![
        Pages::new(Route::Dashboard, "Dashboard"),
        Pages::new(Route::Containers, "Containers"),
        Pages::new(Route::Images, "Images"),
    ]
});

pub struct SideBar {
    idx_page: usize,
    hovered_idx: Option<usize>,
    theme_color: ThemeColor,
}

impl SideBar {
    pub fn new(theme_color: ThemeColor) -> Self {
        Self {
            idx_page: 0,
            hovered_idx: None,
            theme_color,
        }
    }

    pub fn validate_idx(&self, idx_page: usize) -> bool {
        idx_page < PAGES.len()
    }

    pub fn set_active_route(&mut self, route: Route) {
        if let Some(pos) = PAGES.iter().position(|p| p.route() == route) {
            self.idx_page = pos;
        }
    }

    pub fn active_route(&self) -> Route {
        PAGES
            .get(self.idx_page)
            .map(|p| p.route())
            .unwrap_or(Route::Dashboard)
    }

    pub fn next_route(&self) -> Route {
        let next = (self.idx_page + 1) % PAGES.len();
        PAGES[next].route()
    }

    pub fn prev_route(&self) -> Route {
        let prev = if self.idx_page == 0 {
            PAGES.len() - 1
        } else {
            self.idx_page - 1
        };
        PAGES[prev].route()
    }

    fn item_index_at(&self, area: Rect, pos: Position) -> Option<usize> {
        let block = Block::bordered();
        let inner = block.inner(area);
        if !inner.contains(pos) {
            return None;
        }
        // Row 0..1 is header/divider, items start at inner.y + 2
        let first_item_y = inner.y + 2;
        if pos.y < first_item_y {
            return None;
        }
        let idx = (pos.y - first_item_y) as usize;
        if self.validate_idx(idx) {
            Some(idx)
        } else {
            None
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        self.render(area, frame.buffer_mut());
    }
}

impl BaseWidget for SideBar {
    type Event = SideBarEvent;

    fn handle_key(&mut self, key: KeyEvent) -> Option<SideBarEvent> {
        match key.code {
            KeyCode::Tab => Some(SideBarEvent::NavigatePage(self.next_route())),
            KeyCode::BackTab => Some(SideBarEvent::NavigatePage(self.prev_route())),
            _ => None,
        }
    }

    fn handle_mouse(&mut self, area: Rect, mouse: MouseEvent) -> Option<Self::Event> {
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hovered_idx = self.item_index_at(area, pos);
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = self.item_index_at(area, pos) {
                    self.idx_page = idx;
                    return Some(SideBarEvent::NavigatePage(PAGES[idx].route()));
                }
                None
            }
            _ => None,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" TUI-KK ")
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(self.theme_color.border))
            .style(Style::new().bg(self.theme_color.sidebar_bg));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 || inner.width < 4 {
            return;
        }

        let header_rect = Rect::new(inner.x, inner.y, inner.width, 2);
        Paragraph::new(vec![
            Line::from(Span::styled(
                " Navigation",
                Style::new()
                    .fg(self.theme_color.sidebar_section)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " ──────────────",
                Style::new().fg(self.theme_color.border),
            )),
        ])
        .render(header_rect, buf);

        let list_rect = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );

        let items: Vec<ListItem> = PAGES
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let is_active = i == self.idx_page;
                let is_hovered = self.hovered_idx == Some(i);
                let prefix = if is_active { " ▸ " } else { "   " };
                let num = i + 1;
                let label = format!("{prefix}{num}. {}", page.name);

                let style = if is_active {
                    Style::new()
                        .fg(self.theme_color.sidebar_text_active)
                        .bg(self.theme_color.sidebar_active_bg)
                        .add_modifier(Modifier::BOLD)
                } else if is_hovered {
                    Style::new()
                        .fg(self.theme_color.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(self.theme_color.sidebar_text)
                };

                ListItem::new(Line::from(label)).style(style)
            })
            .collect();

        List::new(items).render(list_rect, buf);
    }
}
