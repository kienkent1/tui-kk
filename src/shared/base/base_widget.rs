use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{buffer::Buffer, layout::Rect};

pub trait BaseWidget {
    type Event;

    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Event>;
    fn handle_mouse(&mut self, area: Rect, mouse: MouseEvent) -> Option<Self::Event>;
    fn render(&self, area: Rect, buf: &mut Buffer);
}
