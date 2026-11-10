use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use crate::{
    modules::images::{
        actions::ImageAction, detail_page::ImageDetailPage, image_service::ImageService,
        list_page::ImageListPage,
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::{
        app_services::AppServices,
        config::{AppConfig, QueryMode},
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

enum ImageRoute {
    List,
    Detail(ImageDetailPage),
}

pub struct ImagePage {
    route: ImageRoute,
    list: ImageListPage,
    tx: Tx,
    svc: Arc<ImageService>,
    theme_color: ThemeColor,
}

impl ImagePage {
    pub fn new(services: &'static AppServices, tx: Tx, theme_color: &ThemeColor) -> Self {
        let svc = services.images.clone();
        let app_config = AppConfig::global();
        Self {
            route: ImageRoute::List,
            list: ImageListPage::new(
                svc.clone(),
                tx.clone(),
                app_config.query_mode == QueryMode::Local,
                theme_color,
            ),
            tx,
            svc,
            theme_color: *theme_color,
        }
    }

    fn active_ref(&self) -> &dyn Page {
        match &self.route {
            ImageRoute::List => &self.list,
            ImageRoute::Detail(p) => p,
        }
    }

    fn active(&mut self) -> &mut dyn Page {
        match &mut self.route {
            ImageRoute::List => &mut self.list,
            ImageRoute::Detail(page) => page,
        }
    }

    fn navigate(&mut self, route: ImageRoute) {
        self.active().on_deactivate();
        self.route = route;
        self.active().on_activate();
    }
}

impl Page for ImagePage {
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
        if matches!(self.route, ImageRoute::Detail(_)) && cmd == Command::Back {
            self.navigate(ImageRoute::List);
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
            Action::Images(ImageAction::OpenDetail(id)) => {
                let detail = ImageDetailPage::new(
                    id.clone(),
                    self.svc.clone(),
                    self.tx.clone(),
                    &self.theme_color,
                );
                self.navigate(ImageRoute::Detail(detail));
                return true;
            }
            Action::Images(ImageAction::BackToList) => {
                self.navigate(ImageRoute::List);
                return true;
            }
            _ => {}
        }

        let mut handled = self.list.update(action);
        if let ImageRoute::Detail(d) = &mut self.route {
            handled |= d.update(action);
        }

        handled
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.active().draw(frame, area);
    }
}
