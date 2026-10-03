use std::collections::HashMap;

use crate::{
    modules::containers::ContainerPage,
    shared::base::{
        base_action::Action,
        base_component::{PageBox, Tx},
    },
    tuikk_core::route::Route,
};
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use strum::EnumCount;

pub enum PageMsg {}

#[derive(Default)]
pub struct Router {
    current: Route,
    pages: HashMap<Route, PageBox>,
}

impl Router {
    pub fn new() -> Self {
        let mut pages: HashMap<Route, PageBox> = HashMap::new();

        pages.insert(Route::Containers, Box::new(ContainerPage::new()));

        Self {
            current: Route::default(),
            pages,
        }
    }

    fn active(&mut self) -> &mut PageBox {
        //use expect to check runtime
        self.pages
            .get_mut(&self.current)
            .expect("active route must have a page")
    }

    pub fn activate_current(&mut self, tx: &Tx) {
        self.active().on_activate(tx);
    }

    pub fn navigate(&mut self, route: Route, tx: &Tx) {
        if route == self.current {
            return;
        }
        self.active().on_deactivate();
        self.current = route;
        self.active().on_activate(tx);
    }

    pub fn tick(&mut self) -> bool {
        self.active().tick()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        self.active().handle_key_event(key)
    }

    pub fn dispatch(&mut self, action: Action) -> bool {
        self.active().update(action)
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.active().draw(frame, area);
    }

    pub fn current_route(&self) -> Route {
        self.current
    }
}
