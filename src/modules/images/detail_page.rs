use std::sync::Arc;

use bollard::models::ImageInspect;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect};

use crate::{
    modules::images::{
        actions::ImageAction as IA, constants::ImageDetailTab, image_service::ImageService, ui,
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::{
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

pub struct ImageDetailPage {
    id: String,
    svc: Arc<ImageService>,
    tx: Tx,
    image: ImageInspect,

    active_tab: ImageDetailTab,
    scroll: u16,
    loading: bool,
    error: Option<String>,
    theme_color: ThemeColor,
}

impl ImageDetailPage {
    pub fn new(id: String, svc: Arc<ImageService>, tx: Tx, theme_color: &ThemeColor) -> Self {
        Self {
            id,
            svc,
            tx,
            image: ImageInspect::default(),
            active_tab: ImageDetailTab::default(),
            scroll: 0,
            loading: false,
            error: None,
            theme_color: *theme_color,
        }
    }

    fn fetch(&mut self) {
        self.loading = true;
        self.error = None;
        let (svc, tx, id) = (self.svc.clone(), self.tx.clone(), self.id.clone());
        tokio::spawn(async move {
            let res = svc.get_image(&id).await.map_err(|e| e.to_string());
            let _ = tx.send(Action::Images(IA::DetailLoaded(id, res)));
        });
    }

    pub fn scroll_by(&mut self, delta: i16) {
        if delta < 0 {
            self.scroll = self.scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.scroll = self.scroll.saturating_add(delta as u16);
        }
    }

    pub fn set_tab(&mut self, tab: ImageDetailTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.scroll = 0;
        }
    }
}

impl Page for ImageDetailPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Images
    }

    fn on_activate(&mut self) {
        if self.image.id.is_none() {
            self.fetch();
        }
    }

    fn on_deactivate(&mut self) {}

    fn tick(&mut self) -> bool {
        true
    }

    fn captures_input(&self) -> bool {
        false
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        match cmd {
            Command::Down => self.scroll_by(1),
            Command::Up => self.scroll_by(-1),
            Command::Refresh => self.fetch(),
            _ => {}
        }
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('1') => self.set_tab(ImageDetailTab::Overview),
            KeyCode::Char('2') => self.set_tab(ImageDetailTab::Layers),
            KeyCode::Char('3') => self.set_tab(ImageDetailTab::Config),
            KeyCode::Char('4') => self.set_tab(ImageDetailTab::RawJson),
            KeyCode::Right | KeyCode::Char('l') => self.set_tab(self.active_tab.next()),
            KeyCode::Left | KeyCode::Char('h') => self.set_tab(self.active_tab.prev()),
            KeyCode::PageDown => self.scroll_by(10),
            KeyCode::PageUp => self.scroll_by(-10),
            _ => {}
        }
        None
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.scroll_by(2),
            MouseEventKind::ScrollUp => self.scroll_by(-2),
            _ => {}
        }
        None
    }

    fn update(&mut self, action: &Action) -> bool {
        let Action::Images(IA::DetailLoaded(id, res)) = action else {
            return false;
        };

        if *id != self.id {
            return false;
        }

        self.loading = false;

        match res {
            Ok(image) => {
                self.image = image.clone();
                self.error = None;
            }
            Err(e) => self.error = Some(e.clone()),
        }

        true
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        ui::detail_view::draw(
            frame,
            area,
            &self.image,
            self.active_tab,
            self.scroll,
            self.loading,
            self.error.as_deref(),
            &self.theme_color,
        );
    }
}
