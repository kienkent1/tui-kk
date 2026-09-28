use crate::tuikk_core::route::Route;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};
use strum::EnumCount;

pub enum PageMsg {}

#[derive(Default)]
pub struct Router {
    current: Route,
}
