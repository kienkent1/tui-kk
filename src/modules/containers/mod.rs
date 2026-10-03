mod container_dto;
pub mod container_service;
mod detail_page;
mod error;
mod list_page;
mod constants;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    modules::containers::{detail_page::ContainerDetailPage, list_page::ContainerListPage},
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
};

enum ContainerRoute {
    List,
    Detail(ContainerDetailPage),
}

pub struct ContainerPage {
    route: ContainerRoute,
    list: ContainerListPage,
    tx: Option<Tx>,
}

impl ContainerPage {
    pub fn new() -> Self {
        Self {
            route: ContainerRoute::List,
            list: ContainerListPage::new(),
            tx: None,
        }
    }

    fn active(&mut self) -> &mut dyn Page {
        match &mut self.route {
            ContainerRoute::List => &mut self.list,
            ContainerRoute::Detail(page) => page,
        }
    }

    fn navigate(&mut self, route: ContainerRoute, tx: &Tx) {
        self.active().on_deactivate();
        self.route = route;
        self.active().on_activate(tx);
    }
}

impl Page for ContainerPage {
    fn scope(&self) -> &'static str {
        "containers"
    }

    fn on_activate(&mut self, tx: &Tx) {
        self.tx = Some(tx.clone());
        self.active().on_activate(tx);
    }

    fn on_deactivate(&mut self) {
        self.active().on_deactivate();
    }

    fn tick(&mut self) -> bool {
        self.active().tick()
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        self.active().handle_key_event(key)
    }
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        self.handle_mouse_event(mouse)
    }

    fn update(&mut self, action: Action) -> bool {
        // Bắt action navigate nội bộ trước
        // match &action {
        //     Action::Feature(FeatureAction::Container(ContainerAction::OpenDetail(id))) => {
        //         self.navigate(ContainerRoute::Detail(id.clone()) /* tx? */);
        //         return true;
        //     }
        //     Action::Back => {
        //         self.navigate(ContainerRoute::List /* tx? */);
        //         return true;
        //     }
        //     _ => {}
        // }
        // // Còn lại forward xuống active sub-page
        // self.active().update(action)

        true
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.active().draw(frame, area);
    }
}
