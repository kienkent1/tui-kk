mod detail_page;
mod list_page;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::{
    modules::containers::{detail_page::ContainerDetailPage, list_page::ContainerListPage},
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
};

mod container_service;

enum ContainerRoute {
    List,
    Detail(String),
}

pub struct ContainerPage {
    route: ContainerRoute,
    list: ContainerListPage,
    detail: ContainerDetailPage,
    tx: Option<Tx>,
}

impl ContainerPage {
    pub fn new() -> Self {
        Self {
            route: ContainerRoute::List,
            list: ContainerListPage::new(),
            detail: ContainerDetailPage::new(),
            tx: None,
        }
    }

    fn active(&mut self) -> &mut dyn Page {
        match &self.route {
            ContainerRoute::List => &mut self.list,
            ContainerRoute::Detail(_) => &mut self.detail,
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
