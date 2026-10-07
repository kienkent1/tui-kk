use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

pub trait Widget {
    type Event;

    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Event>;
    fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<Self::Event>;
    fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool);
}
