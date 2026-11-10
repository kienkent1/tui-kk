use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use crate::{
    modules::logs::{
        actions::LogAction,
        detail_page::LogDetailPage,
        list_page::LogListPage,
        log_service::LogService,
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::{
        app_services::AppServices,
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

enum LogRoute {
    List,
    Detail(LogDetailPage),
}

pub struct LogPage {
    route: LogRoute,
    list: LogListPage,
    tx: Tx,
    svc: Arc<LogService>,
    theme_color: ThemeColor,
}

impl LogPage {
    pub fn new(services: &'static AppServices, tx: Tx, theme_color: &ThemeColor) -> Self {
        let svc = services.logs.clone();
        Self {
            route: LogRoute::List,
            list: LogListPage::new(svc.clone(), tx.clone(), theme_color),
            tx,
            svc,
            theme_color: *theme_color,
        }
    }

    fn active_ref(&self) -> &dyn Page {
        match &self.route {
            LogRoute::List => &self.list,
            LogRoute::Detail(p) => p,
        }
    }

    fn active(&mut self) -> &mut dyn Page {
        match &mut self.route {
            LogRoute::List => &mut self.list,
            LogRoute::Detail(page) => page,
        }
    }

    fn navigate(&mut self, route: LogRoute) {
        self.active().on_deactivate();
        self.route = route;
        self.active().on_activate();
    }
}

impl Page for LogPage {
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
        if matches!(self.route, LogRoute::Detail(_)) && cmd == Command::Back {
            self.navigate(LogRoute::List);
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
        match action {
            Action::LogsPage(LogAction::OpenDetail(file_name)) => {
                let detail = LogDetailPage::new(
                    file_name.clone(),
                    self.svc.clone(),
                    self.tx.clone(),
                    &self.theme_color,
                );
                self.navigate(LogRoute::Detail(detail));
                return true;
            }
            Action::LogsPage(LogAction::BackToList) => {
                self.navigate(LogRoute::List);
                return true;
            }
            Action::LogsPage(LogAction::FileDeleted(deleted_name)) => {
                let should_go_back = match &self.route {
                    LogRoute::Detail(d) => d.file_name() == deleted_name,
                    LogRoute::List => false,
                };
                if should_go_back {
                    self.navigate(LogRoute::List);
                    return true;
                }
            }
            _ => {}
        }

        let mut handled = self.list.update(action);
        if let LogRoute::Detail(d) = &mut self.route {
            handled |= d.update(action);
        }

        handled
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.active().draw(frame, area);
    }
}
