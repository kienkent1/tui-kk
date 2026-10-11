use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, buffer::Buffer, layout::Rect};
use std::sync::LazyLock;

use crate::{
    shared::base::base_widget::BaseWidget,
    tuikk_core::route::{Pages, Route},
};

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
}

impl SideBar {
    pub fn new() -> Self {
        Self { idx_page: 0 }
    }

    fn validate_idx(&self, idx_page: usize) -> bool {
        if idx_page >= PAGES.len() {
            return false;
        }

        true
    }
}

impl BaseWidget for SideBar {
    type Event = SideBarEvent;
    fn handle_key(&mut self, key: KeyEvent) -> Option<SideBarEvent> {
        None
    }
    fn handle_mouse(&mut self, area: Rect, mouse: MouseEvent) -> Option<Self::Event> {
        None
    }
    fn render(&self, area: Rect, buf: &mut Buffer) {}
}
