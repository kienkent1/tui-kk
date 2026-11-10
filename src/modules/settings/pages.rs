use std::{path::PathBuf, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
};

use crate::{
    modules::settings::{
        actions::SettingsAction as SA,
        constants::{DockerField, GeneralField, SettingsTab},
        settings_dto::{KeybindingRow, SettingsDraft},
        settings_service::SettingsService,
        ui::{self, footer::FooterButtonAreas},
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
    },
    tuikk_core::{
        app_services::AppServices,
        config::AppConfig,
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

pub struct SettingsPage {
    svc: Arc<SettingsService>,
    tx: Tx,
    theme_color: ThemeColor,
    config_path: PathBuf,

    active_tab: SettingsTab,
    general_focus: GeneralField,
    docker_focus: DockerField,
    editing_text: bool,

    draft: SettingsDraft,
    dirty: bool,

    keybindings: Vec<KeybindingRow>,
    kb_cursor: usize,
    kb_offset: usize,
    kb_viewport: usize,

    footer_btns: FooterButtonAreas,
    status_msg: Option<String>,
    error_msg: Option<String>,
}

impl SettingsPage {
    pub fn new(services: &'static AppServices, tx: Tx, theme_color: &ThemeColor) -> Self {
        let svc = services.settings.clone();
        let cfg = AppConfig::global();
        let draft = SettingsDraft::from_config(&cfg);
        let keybindings = SettingsService::extract_keybindings(&cfg.keybindings);
        Self::with_draft(svc, draft, keybindings, tx, theme_color)
    }

    pub fn with_draft(
        svc: Arc<SettingsService>,
        draft: SettingsDraft,
        keybindings: Vec<KeybindingRow>,
        tx: Tx,
        theme_color: &ThemeColor,
    ) -> Self {
        let config_path = svc.config_file_path();
        Self {
            svc,
            tx,
            theme_color: *theme_color,
            config_path,
            active_tab: SettingsTab::General,
            general_focus: GeneralField::Theme,
            docker_focus: DockerField::ConnectionType,
            editing_text: false,
            draft,
            dirty: false,
            keybindings,
            kb_cursor: 0,
            kb_offset: 0,
            kb_viewport: 15,
            footer_btns: FooterButtonAreas::default(),
            status_msg: None,
            error_msg: None,
        }
    }

    pub fn active_tab(&self) -> SettingsTab {
        self.active_tab
    }

    pub fn draft(&self) -> &SettingsDraft {
        &self.draft
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_tab(&mut self, tab: SettingsTab) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.editing_text = false;
        }
    }

    fn move_focus(&mut self, down: bool) {
        match self.active_tab {
            SettingsTab::General => {
                self.general_focus = if down {
                    self.general_focus.next()
                } else {
                    self.general_focus.prev()
                };
            }
            SettingsTab::Docker => {
                self.docker_focus = if down {
                    self.docker_focus.next()
                } else {
                    self.docker_focus.prev()
                };
            }
            SettingsTab::Keybindings => {
                let delta = if down { 1 } else { -1 };
                self.move_kb_cursor(delta);
            }
        }
    }

    fn move_kb_cursor(&mut self, delta: isize) {
        let len = self.keybindings.len();
        if len == 0 {
            self.kb_cursor = 0;
            self.kb_offset = 0;
            return;
        }
        self.kb_cursor = (self.kb_cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        let h = self.kb_viewport.max(1);
        if self.kb_cursor < self.kb_offset {
            self.kb_offset = self.kb_cursor;
        } else if self.kb_cursor >= self.kb_offset + h {
            self.kb_offset = self.kb_cursor - h + 1;
        }
        self.kb_offset = self.kb_offset.min(len.saturating_sub(h));
    }

    pub fn step_focused_value(&mut self, forward: bool) {
        match self.active_tab {
            SettingsTab::General => match self.general_focus {
                GeneralField::Theme => {
                    self.draft.cycle_theme();
                    self.dirty = true;
                }
                GeneralField::IconSet => {
                    self.draft.cycle_icon_set(forward);
                    self.dirty = true;
                }
                GeneralField::QueryMode => {
                    self.draft.cycle_query_mode();
                    self.dirty = true;
                }
                GeneralField::LogLevel => {
                    self.draft.cycle_log_level(forward);
                    self.dirty = true;
                }
                GeneralField::FrameRate => {
                    self.draft.adjust_frame_rate(if forward { 5.0 } else { -5.0 });
                    self.dirty = true;
                }
                GeneralField::TickRate => {
                    self.draft.adjust_tick_rate(if forward { 1.0 } else { -1.0 });
                    self.dirty = true;
                }
                GeneralField::LogDir => {}
            },
            SettingsTab::Docker => match self.docker_focus {
                DockerField::ConnectionType => {
                    self.draft.cycle_conn_type(forward);
                    self.dirty = true;
                }
                DockerField::Timeout => {
                    self.draft.adjust_timeout(if forward { 10 } else { -10 });
                    self.dirty = true;
                }
                DockerField::AddrOrPath | DockerField::ClientVersion => {}
            },
            SettingsTab::Keybindings => {}
        }
    }

    fn is_text_field_focused(&self) -> bool {
        matches!(
            (self.active_tab, self.general_focus, self.docker_focus),
            (SettingsTab::General, GeneralField::LogDir, _)
                | (SettingsTab::Docker, _, DockerField::AddrOrPath)
                | (SettingsTab::Docker, _, DockerField::ClientVersion)
        )
    }

    fn active_text_mut(&mut self) -> Option<&mut String> {
        match self.active_tab {
            SettingsTab::General if self.general_focus == GeneralField::LogDir => {
                Some(&mut self.draft.log_dir)
            }
            SettingsTab::Docker if self.docker_focus == DockerField::AddrOrPath => {
                Some(&mut self.draft.addr_or_path)
            }
            SettingsTab::Docker if self.docker_focus == DockerField::ClientVersion => {
                Some(&mut self.draft.client_version)
            }
            _ => None,
        }
    }

    fn save_config(&mut self) {
        match self.svc.save_draft(&self.draft) {
            Ok(updated) => {
                self.theme_color = updated.theme_color;
                self.dirty = false;
                self.error_msg = None;
                self.status_msg = Some(format!(
                    "Saved configuration to '{}'",
                    self.config_path.display()
                ));
            }
            Err(e) => {
                self.error_msg = Some(e);
            }
        }
    }

    fn reload_config(&mut self) {
        match self.svc.reload_from_disk() {
            Ok(draft) => {
                self.draft = draft;
                self.dirty = false;
                self.error_msg = None;
                self.status_msg = Some("Reloaded configuration from disk".to_owned());
            }
            Err(e) => {
                self.error_msg = Some(e);
            }
        }
    }

    pub fn restore_defaults(&mut self) {
        self.draft = SettingsDraft::default();
        self.dirty = true;
        self.error_msg = None;
        self.status_msg = Some("Restored default settings (press 'w' to save)".to_owned());
    }

    fn spawn_ping_docker(&mut self) {
        self.error_msg = None;
        self.status_msg = Some("Testing Docker daemon connection…".to_owned());
        let (svc, draft, tx) = (self.svc.clone(), self.draft.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = svc.reload_and_ping_docker(&draft).await;
            let _ = tx.send(Action::Settings(SA::PingCompleted(res)));
        });
    }
}

impl Page for SettingsPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Settings
    }

    fn captures_input(&self) -> bool {
        self.editing_text
    }

    fn on_activate(&mut self) {}

    fn on_deactivate(&mut self) {
        self.editing_text = false;
    }

    fn tick(&mut self) -> bool {
        true
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        match cmd {
            Command::Down => self.move_focus(true),
            Command::Up => self.move_focus(false),
            Command::Refresh => return Some(Action::Settings(SA::ReloadFromDisk)),
            Command::Select => {
                if self.is_text_field_focused() {
                    self.editing_text = !self.editing_text;
                } else {
                    self.step_focused_value(true);
                }
            }
            Command::Back => {
                self.editing_text = false;
            }
            _ => {}
        }
        None
    }

    fn handle_paste(&mut self, text: &str) -> Option<Action> {
        if self.editing_text {
            if let Some(buf) = self.active_text_mut() {
                buf.extend(text.chars().filter(|c| !c.is_control()));
                self.dirty = true;
            }
        }
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if self.editing_text {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.editing_text = false;
                }
                KeyCode::Backspace => {
                    if let Some(buf) = self.active_text_mut() {
                        buf.pop();
                        self.dirty = true;
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(buf) = self.active_text_mut() {
                        buf.push(c);
                        self.dirty = true;
                    }
                }
                _ => {}
            }
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('d') | KeyCode::Char('D'))
        {
            return Some(Action::Settings(SA::RestoreDefaults));
        }

        match key.code {
            KeyCode::Char('1') => self.set_tab(SettingsTab::General),
            KeyCode::Char('2') => self.set_tab(SettingsTab::Docker),
            KeyCode::Char('3') => self.set_tab(SettingsTab::Keybindings),
            KeyCode::Char('h') => self.set_tab(self.active_tab.prev()),
            KeyCode::Char('l') => self.set_tab(self.active_tab.next()),
            KeyCode::Right | KeyCode::Char(' ') | KeyCode::Char('+') | KeyCode::Char('=') => {
                self.step_focused_value(true);
            }
            KeyCode::Left | KeyCode::Char('-') => {
                self.step_focused_value(false);
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                return Some(Action::Settings(SA::Save));
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                return Some(Action::Settings(SA::PingDocker));
            }
            _ => {}
        }
        None
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.footer_btns.save_btn.contains(pos) {
                    return Some(Action::Settings(SA::Save));
                }
                if self.footer_btns.reload_btn.contains(pos) {
                    return Some(Action::Settings(SA::ReloadFromDisk));
                }
                if self.footer_btns.defaults_btn.contains(pos) {
                    return Some(Action::Settings(SA::RestoreDefaults));
                }
                if self.footer_btns.ping_btn.contains(pos) {
                    return Some(Action::Settings(SA::PingDocker));
                }
            }
            MouseEventKind::ScrollDown => self.move_focus(true),
            MouseEventKind::ScrollUp => self.move_focus(false),
            _ => {}
        }
        None
    }

    fn update(&mut self, action: &Action) -> bool {
        let Action::Settings(sa) = action else {
            return false;
        };

        match sa {
            SA::Save => {
                self.save_config();
                true
            }
            SA::ReloadFromDisk => {
                self.reload_config();
                true
            }
            SA::RestoreDefaults => {
                self.restore_defaults();
                true
            }
            SA::PingDocker => {
                self.spawn_ping_docker();
                true
            }
            SA::PingCompleted(res) => {
                match res {
                    Ok(msg) => {
                        self.error_msg = None;
                        self.status_msg = Some(msg.clone());
                    }
                    Err(e) => {
                        self.error_msg = Some(e.clone());
                    }
                }
                true
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [header_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(6),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .areas(area);

        ui::header::draw(
            frame,
            header_area,
            &self.config_path,
            self.active_tab,
            self.dirty,
            self.status_msg.as_deref(),
            self.error_msg.as_deref(),
            &self.theme_color,
        );

        match self.active_tab {
            SettingsTab::General => {
                ui::general_tab::draw(
                    frame,
                    body_area,
                    &self.draft,
                    self.general_focus,
                    self.editing_text,
                    &self.theme_color,
                );
            }
            SettingsTab::Docker => {
                ui::docker_tab::draw(
                    frame,
                    body_area,
                    &self.draft,
                    self.docker_focus,
                    self.editing_text,
                    &self.theme_color,
                );
            }
            SettingsTab::Keybindings => {
                self.kb_viewport = ui::keybindings_tab::draw(
                    frame,
                    body_area,
                    &self.keybindings,
                    self.kb_cursor,
                    self.kb_offset,
                    &self.theme_color,
                );
            }
        }

        ui::footer::draw(frame, footer_area, &mut self.footer_btns, &self.theme_color);
    }
}

