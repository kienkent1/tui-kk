use color_eyre::config::Frame;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;

pub trait Widget {
    type Event;

    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Event>;
    fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool);
}
