use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect};

use crate::{
    modules::logs::{
        actions::LogAction as LA,
        constants::LogDetailTab,
        log_dto::LogEntry,
        log_service::LogService,
        ui,
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

pub struct LogDetailPage {
    file_name: String,
    svc: Arc<LogService>,
    tx: Tx,
    entries: Vec<LogEntry>,

    active_tab: LogDetailTab,
    scroll: u16,
    loading: bool,
    error: Option<String>,
    theme_color: ThemeColor,
}

impl LogDetailPage {
    pub fn new(
        file_name: String,
        svc: Arc<LogService>,
        tx: Tx,
        theme_color: &ThemeColor,
    ) -> Self {
        Self {
            file_name,
            svc,
            tx,
            entries: Vec::new(),
            active_tab: LogDetailTab::default(),
            scroll: 0,
            loading: false,
            error: None,
            theme_color: *theme_color,
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    fn fetch(&mut self) {
        self.loading = true;
        self.error = None;
        let (svc, tx, file_name) = (self.svc.clone(), self.tx.clone(), self.file_name.clone());
        tokio::task::spawn_blocking(move || {
            let res = svc.read_log_entries(&file_name).map_err(|e| e.to_string());
            let _ = tx.send(Action::LogsPage(LA::DetailLoaded(file_name, res)));
        });
    }

    pub fn scroll_by(&mut self, delta: i16) {
        if delta < 0 {
            self.scroll = self.scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.scroll = self.scroll.saturating_add(delta as u16);
        }
    }

    pub fn set_tab(&mut self, tab: LogDetailTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.scroll = 0;
        }
    }
}

impl Page for LogDetailPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Logs
    }

    fn on_activate(&mut self) {
        if self.entries.is_empty() && !self.loading {
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
            Command::Delete => {
                return Some(Action::LogsPage(LA::DeleteFile(self.file_name.clone())));
            }
            _ => {}
        }
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('1') => self.set_tab(LogDetailTab::WarnAndError),
            KeyCode::Char('2') => self.set_tab(LogDetailTab::ErrorsOnly),
            KeyCode::Char('3') => self.set_tab(LogDetailTab::All),
            KeyCode::Char('4') => self.set_tab(LogDetailTab::Raw),
            KeyCode::Right | KeyCode::Char('l') => self.set_tab(self.active_tab.next()),
            KeyCode::Left | KeyCode::Char('h') => self.set_tab(self.active_tab.prev()),
            KeyCode::PageDown => self.scroll_by(10),
            KeyCode::PageUp => self.scroll_by(-10),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                return Some(Action::LogsPage(LA::ClearFile(self.file_name.clone())));
            }
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
        match action {
            Action::LogsPage(LA::DetailLoaded(file_name, res)) if *file_name == self.file_name => {
                self.loading = false;
                match res {
                    Ok(entries) => {
                        self.entries = entries.clone();
                        self.error = None;
                    }
                    Err(e) => self.error = Some(e.clone()),
                }
                true
            }
            Action::LogsPage(LA::ActionCompleted(Ok(name))) if *name == self.file_name => {
                self.fetch();
                true
            }
            _ => false,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        ui::detail_view::draw(
            frame,
            area,
            &self.file_name,
            &self.entries,
            self.active_tab,
            self.scroll,
            self.loading,
            self.error.as_deref(),
            &self.theme_color,
        );
    }
}
