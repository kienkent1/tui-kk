use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use crate::{
    modules::containers::container_service::ContainerService,
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::key_map::{Command, KeyScope},
};

pub struct ContainerDetailPage {
    id: String,
}

impl ContainerDetailPage {
    pub fn new(id: String, svc: Arc<ContainerService>, tx: Tx) -> Self {
        Self { id: id }
    }
}
impl Page for ContainerDetailPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Containers
    }
    fn on_activate(&mut self) {}
    fn on_deactivate(&mut self) {}
    fn tick(&mut self) -> bool {
        true
    }
    fn captures_input(&self) -> bool {
        true
    }
    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        None
    }
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        None
    }
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        None
    }
    fn update(&mut self, action: &Action) -> bool {
        true
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {}
}
