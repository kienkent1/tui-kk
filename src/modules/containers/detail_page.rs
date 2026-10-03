
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use crate::shared::base::{base_action::Action, base_component::{Page, Tx}};

pub struct ContainerDetailPage {
    id: String,
}

impl ContainerDetailPage{
    pub fn new(id: String) -> Self {
        Self { id: id }
    }
}
impl Page for ContainerDetailPage {

    fn scope(&self) -> &'static str{
        ""
    }
    fn on_activate(&mut self, tx: &Tx){

    }
    fn on_deactivate(&mut self){}
    fn tick(&mut self) -> bool{true}
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action>{
        None
    }
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action>{None}
    fn update(&mut self, action: Action) -> bool{true}
    fn draw(&mut self, frame: &mut Frame, area: Rect){}
    fn handle_action(&mut self, action: Action) -> bool {
        false
    }
}
