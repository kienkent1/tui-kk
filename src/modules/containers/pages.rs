use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};
use tokio::task::JoinHandle;

use crate::{
    modules::containers::{
        actions::ContainerAction, container_service::ContainerService,
        detail_page::ContainerDetailPage, list_page::ContainerListPage,
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::{
        app_services::AppServices,
        config::{AppConfig, QueryMode},
        key_map::{Command, KeyScope},
    },
};

enum ContainerRoute {
    List,
    Detail(ContainerDetailPage),
}

pub struct ContainerPage {
    route: ContainerRoute,
    list: ContainerListPage,
    tx: Tx,
    tasks: Vec<JoinHandle<()>>,
    svc: Arc<ContainerService>,
}

impl ContainerPage {
    pub fn new(services: &'static AppServices, tx: Tx) -> Self {
        let svc = services.containers.clone();
        let app_config = AppConfig::global();
        Self {
            route: ContainerRoute::List,
            list: ContainerListPage::new(
                svc.clone(),
                tx.clone(),
                app_config.query_mode == QueryMode::Local,
            ),
            tx: tx,
            tasks: Vec::new(),
            svc: svc.clone(),
        }
    }

    fn active_ref(&self) -> &dyn Page {
        match &self.route {
            ContainerRoute::List => &self.list,
            ContainerRoute::Detail(p) => p,
        }
    }

    fn active(&mut self) -> &mut dyn Page {
        match &mut self.route {
            ContainerRoute::List => &mut self.list,
            ContainerRoute::Detail(page) => page,
        }
    }

    fn navigate(&mut self, route: ContainerRoute) {
        self.active().on_deactivate();
        self.route = route;
        self.active().on_activate();
    }
}

impl Page for ContainerPage {
    fn scope(&self) -> KeyScope {
        self.active_ref().scope()
    }

    fn captures_input(&self) -> bool {
        self.active_ref().captures_input()
    }

    fn on_activate(&mut self) {
        self.active().on_activate();
    }
    fn on_deactivate(&mut self) {
        self.active().on_deactivate();
    }
    fn tick(&mut self) -> bool {
        self.active().tick()
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        if matches!(self.route, ContainerRoute::Detail(_)) && cmd == Command::Back {
            self.navigate(ContainerRoute::List);
            return None;
        }

        self.active().handle_command(cmd)
    }
    fn handle_paste(&mut self, text: &str) -> Option<Action> {
        self.active().handle_paste(text)
    }
    fn handle_key_event(&mut self, k: KeyEvent) -> Option<Action> {
        self.active().handle_key_event(k)
    }
    fn handle_mouse_event(&mut self, m: MouseEvent) -> Option<Action> {
        self.active().handle_mouse_event(m)
    }

    fn update(&mut self, action: &Action) -> bool {
        if let Action::Containers(ContainerAction::OpenDetail(id)) = action {
            let detail = ContainerDetailPage::new(id.clone(), self.svc.clone(), self.tx.clone());
            self.navigate(ContainerRoute::Detail(detail));
            return true;
        }

        let mut handled = self.list.update(action);
        if let ContainerRoute::Detail(d) = &mut self.route {
            handled |= d.update(action);
        }

        handled
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.active().draw(frame, area);
    }
}
