use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::{
    modules::{
        containers::pages::ContainerPage,
        dashboard::pages::DashboardPage,
        images::pages::ImagePage,
        logs::pages::LogPage,
        settings::pages::SettingsPage,
    },
    shared::{
        base::{
            base_action::Action,
            base_component::{PageBox, Tx},
            base_widget::BaseWidget,
        },
        widgets::side_bar::{SideBar, SideBarEvent},
    },
    tuikk_core::{
        app_services::AppServices,
        config::AppConfig,
        key_map::{Command, KeyScope},
        route::Route,
    },
};

pub struct Router {
    current: Route,
    pages: HashMap<Route, PageBox>,
    sidebar: SideBar,
    sidebar_area: Rect,
}

impl Router {
    pub fn new(tx: &Tx) -> Self {
        let mut pages: HashMap<Route, PageBox> = HashMap::new();
        let services = AppServices::get();
        let app_config = AppConfig::global();

        pages.insert(
            Route::Dashboard,
            Box::new(DashboardPage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        pages.insert(
            Route::Containers,
            Box::new(ContainerPage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        pages.insert(
            Route::Images,
            Box::new(ImagePage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        pages.insert(
            Route::Logs,
            Box::new(LogPage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        pages.insert(
            Route::Settings,
            Box::new(SettingsPage::new(
                services,
                tx.clone(),
                &app_config.theme_color,
            )),
        );

        let current = Route::default();
        let mut sidebar = SideBar::new(app_config.theme_color);
        sidebar.set_active_route(current);

        Self {
            current,
            pages,
            sidebar,
            sidebar_area: Rect::default(),
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
        if let Some(SideBarEvent::NavigatePage(route)) =
            self.sidebar.handle_mouse(self.sidebar_area, mouse)
        {
            if self.pages.contains_key(&route) {
                return Some(Action::Navigate(route));
            }
        }
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
        self.sidebar.set_active_route(route);
        self.current_page_mut().on_activate();
    }

    pub fn next_route(&self) -> Route {
        self.sidebar.next_route()
    }

    pub fn prev_route(&self) -> Route {
        self.sidebar.prev_route()
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
        let [sidebar_area, content_area] =
            Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)]).areas(area);

        self.sidebar_area = sidebar_area;
        self.sidebar.draw(frame, sidebar_area);
        self.current_page_mut().draw(frame, content_area);
    }

    pub fn current_route(&self) -> Route {
        self.current
    }
}
