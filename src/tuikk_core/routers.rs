use std::collections::HashMap;

use crate::{
    modules::containers::pages::ContainerPage,
    shared::base::{
        base_action::Action,
        base_component::{PageBox, Tx},
    },
    tuikk_core::{
        app_services::AppServices,
        config::AppConfig,
        key_map::{Command, KeyScope},
        route::Route,
    },
};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};
use strum::EnumCount;

pub enum PageMsg {}

#[derive(Default)]
pub struct Router {
    current: Route,
    pages: HashMap<Route, PageBox>,
}

impl Router {
    pub fn new(tx: &Tx) -> Self {
        let mut pages: HashMap<Route, PageBox> = HashMap::new();
        let services = AppServices::get();
        let app_config = AppConfig::global();

        pages.insert(
            Route::Containers,
            Box::new(ContainerPage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        Self {
            current: Route::default(),
            pages,
        }
    }

    // ---------- access current page ----------

    fn current_page(&self) -> &PageBox {
        self.pages
            .get(&self.current)
            .expect("active route must have a page")
    }

    fn current_page_mut(&mut self) -> &mut PageBox {
        self.pages
            .get_mut(&self.current)
            .expect("active route must have a page")
    }

    // ---------- keymap ----------

    pub fn scope(&self) -> KeyScope {
        self.current_page().scope()
    }
    pub fn captures_input(&self) -> bool {
        self.current_page().captures_input()
    }
    pub fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        self.current_page_mut().handle_command(cmd)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        self.current_page_mut().handle_key_event(key)
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<Action> {
        self.current_page_mut().handle_mouse_event(mouse)
    }

    pub fn handle_paste(&mut self, s: &str) -> Option<Action> {
        self.current_page_mut().handle_paste(s)
    }

    // ---------- lifecycle ----------

    pub fn activate_current(&mut self) {
        self.current_page_mut().on_activate();
    }

    pub fn navigate(&mut self, route: Route) {
        if route == self.current || !self.pages.contains_key(&route) {
            return;
        }
        self.current_page_mut().on_deactivate();
        self.current = route;
        self.current_page_mut().on_activate();
    }

    pub fn tick(&mut self) -> bool {
        self.current_page_mut().tick()
    }

    pub fn dispatch(&mut self, action: Action) {
        for page in self.pages.values_mut() {
            page.update(&action);
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.current_page_mut().draw(frame, area);
    }

    pub fn current_route(&self) -> Route {
        self.current
    }
}
