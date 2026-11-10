use std::sync::Arc;

use bollard::models::ImageInspect;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Position, Rect},
};

use crate::{
    modules::images::{
        actions::ImageAction as IA,
        constants::RunField,
        image_dto::{
            EnvVarEntry, RunImageRequest, expand_tilde_path, merge_env_vars,
            parse_env_content_with_source, parse_env_file, parse_port_mappings,
            parse_volume_binds,
        },
        image_service::ImageService,
        ui::{
            self,
            run_view::{RunViewAreas, RunViewProps},
        },
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

pub struct ImageRunPage {
    image_id: String,
    image_ref: String,
    svc: Arc<ImageService>,
    tx: Tx,
    theme_color: ThemeColor,

    exposed_ports_hint: Vec<String>,
    image_default_env: Vec<EnvVarEntry>,

    focus: RunField,
    container_name: String,
    ports_input: String,
    volumes_input: String,
    env_file_path: String,
    env_key_input: String,
    env_val_input: String,
    env_vars: Vec<EnvVarEntry>,

    env_cursor: usize,
    env_offset: usize,
    areas: RunViewAreas,

    loading: bool,
    status_msg: Option<String>,
    error_msg: Option<String>,
}

impl ImageRunPage {
    pub fn new(
        image_id: String,
        image_ref: String,
        svc: Arc<ImageService>,
        tx: Tx,
        theme_color: &ThemeColor,
    ) -> Self {
        Self {
            image_id,
            image_ref,
            svc,
            tx,
            theme_color: *theme_color,
            exposed_ports_hint: Vec::new(),
            image_default_env: Vec::new(),
            focus: RunField::ContainerName,
            container_name: String::new(),
            ports_input: String::new(),
            volumes_input: String::new(),
            env_file_path: ".env".to_owned(),
            env_key_input: String::new(),
            env_val_input: String::new(),
            env_vars: Vec::new(),
            env_cursor: 0,
            env_offset: 0,
            areas: RunViewAreas::default(),
            loading: false,
            status_msg: None,
            error_msg: None,
        }
    }

    fn fetch_image_metadata(&mut self) {
        let (svc, tx, id) = (self.svc.clone(), self.tx.clone(), self.image_id.clone());
        tokio::spawn(async move {
            let res = svc.get_image(&id).await.map_err(|e| e.to_string());
            let _ = tx.send(Action::Images(IA::RunImageInspectLoaded(id, res)));
        });
    }

    fn apply_image_inspect(&mut self, inspect: &ImageInspect) {
        if let Some(cfg) = &inspect.config {
            if let Some(exposed) = &cfg.exposed_ports {
                let mut ports = exposed.clone();
                ports.sort();
                self.exposed_ports_hint = ports;
            }
            if let Some(env_list) = &cfg.env {
                let joined = env_list.join("\n");
                self.image_default_env = parse_env_content_with_source(&joined, "image");
            }
        }
    }

    pub fn load_env_from_path(&mut self) {
        let raw = self.env_file_path.trim();
        if raw.is_empty() {
            self.error_msg = Some("Please enter a .env file path".to_owned());
            return;
        }
        let path = expand_tilde_path(raw);
        match parse_env_file(&path) {
            Ok(entries) => {
                let count = entries.len();
                merge_env_vars(&mut self.env_vars, entries);
                self.clamp_env_cursor();
                self.error_msg = None;
                self.status_msg = Some(format!(
                    "Loaded {count} variable(s) from '{}'",
                    path.display()
                ));
            }
            Err(e) => {
                self.error_msg = Some(e);
            }
        }
    }

    pub fn import_image_default_env(&mut self) {
        if self.image_default_env.is_empty() {
            self.error_msg = Some("Image has no default environment variables".to_owned());
            return;
        }
        let count = self.image_default_env.len();
        merge_env_vars(&mut self.env_vars, self.image_default_env.clone());
        self.clamp_env_cursor();
        self.error_msg = None;
        self.status_msg = Some(format!("Imported {count} default variable(s) from image"));
    }

    pub fn add_manual_env(&mut self) {
        let raw_key = self.env_key_input.trim().to_owned();
        if raw_key.is_empty() {
            self.error_msg = Some("Environment variable key cannot be empty".to_owned());
            return;
        }

        // Support entering `KEY=VALUE` directly in the Key box
        if raw_key.contains('=') && self.env_val_input.is_empty() {
            let parsed = parse_env_content_with_source(&raw_key, "manual");
            if parsed.is_empty() {
                self.error_msg = Some("Invalid KEY=VALUE format".to_owned());
                return;
            }
            let summary = parsed
                .iter()
                .map(|e| e.key.clone())
                .collect::<Vec<_>>()
                .join(", ");
            merge_env_vars(&mut self.env_vars, parsed);
            self.env_key_input.clear();
            self.clamp_env_cursor();
            self.error_msg = None;
            self.status_msg = Some(format!("Added env: {summary}"));
            return;
        }

        if raw_key.contains(char::is_whitespace) {
            self.error_msg =
                Some("Environment variable key must not contain spaces".to_owned());
            return;
        }

        let val = self.env_val_input.clone();
        merge_env_vars(
            &mut self.env_vars,
            vec![EnvVarEntry::new(raw_key.clone(), val, "manual")],
        );
        self.env_key_input.clear();
        self.env_val_input.clear();
        self.focus = RunField::EnvKey;
        self.clamp_env_cursor();
        self.error_msg = None;
        self.status_msg = Some(format!("Set env variable '{raw_key}'"));
    }

    pub fn edit_selected_env(&mut self) {
        if let Some(entry) = self.env_vars.get(self.env_cursor) {
            self.env_key_input = entry.key.clone();
            self.env_val_input = entry.value.clone();
            self.focus = RunField::EnvValue;
            self.error_msg = None;
            self.status_msg = Some(format!("Editing '{}'", entry.key));
        }
    }

    pub fn remove_selected_env(&mut self) {
        if self.env_cursor < self.env_vars.len() {
            let removed = self.env_vars.remove(self.env_cursor);
            self.clamp_env_cursor();
            self.error_msg = None;
            self.status_msg = Some(format!("Removed '{}'", removed.key));
        }
    }

    pub fn clear_all_env(&mut self) {
        self.env_vars.clear();
        self.env_cursor = 0;
        self.env_offset = 0;
        self.error_msg = None;
        self.status_msg = Some("Cleared all environment variables".to_owned());
    }

    pub fn move_env_cursor(&mut self, delta: isize) {
        let len = self.env_vars.len();
        if len == 0 {
            self.env_cursor = 0;
            self.env_offset = 0;
            return;
        }
        self.env_cursor = (self.env_cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.clamp_env_cursor();
    }

    fn clamp_env_cursor(&mut self) {
        let len = self.env_vars.len();
        if len == 0 {
            self.env_cursor = 0;
            self.env_offset = 0;
            return;
        }
        let h = self.areas.env_viewport.max(1);
        self.env_cursor = self.env_cursor.min(len - 1);
        if self.env_cursor < self.env_offset {
            self.env_offset = self.env_cursor;
        } else if self.env_cursor >= self.env_offset + h {
            self.env_offset = self.env_cursor - h + 1;
        }
        self.env_offset = self.env_offset.min(len.saturating_sub(h));
    }

    pub fn build_run_request(&self) -> RunImageRequest {
        let name = self.container_name.trim();
        RunImageRequest {
            image: self.image_ref.clone(),
            container_name: if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            },
            ports: parse_port_mappings(&self.ports_input),
            volumes: parse_volume_binds(&self.volumes_input),
            env_vars: self.env_vars.clone(),
        }
    }

    fn spawn_run(&mut self, req: RunImageRequest) {
        self.loading = true;
        self.error_msg = None;
        self.status_msg = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = svc.run_image(&req).await.map_err(|e| e.to_string());
            let _ = tx.send(Action::Images(IA::RunCompleted(res)));
        });
    }

    fn active_text_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            RunField::ContainerName => Some(&mut self.container_name),
            RunField::Ports => Some(&mut self.ports_input),
            RunField::Volumes => Some(&mut self.volumes_input),
            RunField::EnvFilePath => Some(&mut self.env_file_path),
            RunField::EnvKey => Some(&mut self.env_key_input),
            RunField::EnvValue => Some(&mut self.env_val_input),
            RunField::EnvList | RunField::RunButton => None,
        }
    }
}

impl Page for ImageRunPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Images
    }

    fn captures_input(&self) -> bool {
        true
    }

    fn on_activate(&mut self) {
        self.fetch_image_metadata();
    }

    fn on_deactivate(&mut self) {}

    fn tick(&mut self) -> bool {
        true
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        if cmd == Command::Back {
            return Some(Action::Images(IA::BackToList));
        }
        None
    }

    fn handle_paste(&mut self, text: &str) -> Option<Action> {
        // If pasting multi-line `.env` content into EnvKey or EnvValue, parse all lines automatically!
        if matches!(self.focus, RunField::EnvKey | RunField::EnvValue | RunField::EnvList)
            && (text.contains('\n') || (self.focus == RunField::EnvList && text.contains('=')))
        {
            let entries = parse_env_content_with_source(text, "paste");
            if !entries.is_empty() {
                let count = entries.len();
                merge_env_vars(&mut self.env_vars, entries);
                self.clamp_env_cursor();
                self.error_msg = None;
                self.status_msg = Some(format!("Parsed & added {count} variable(s) from paste"));
                return None;
            }
        }

        if let Some(buf) = self.active_text_mut() {
            buf.extend(text.chars().filter(|c| !c.is_control()));
        }
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if key.code == KeyCode::Esc {
            return Some(Action::Images(IA::BackToList));
        }

        // Global Run Container shortcuts inside RunPage: Ctrl+R or F5
        if (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('r') | KeyCode::Char('R')))
            || key.code == KeyCode::F(5)
        {
            return Some(Action::Images(IA::RunContainer(self.build_run_request())));
        }

        // Ctrl+D imports default image env
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('d') | KeyCode::Char('D'))
        {
            self.import_image_default_env();
            return None;
        }

        match key.code {
            KeyCode::Tab => {
                self.focus = self.focus.next();
                return None;
            }
            KeyCode::BackTab => {
                self.focus = self.focus.prev();
                return None;
            }
            _ => {}
        }

        match self.focus {
            RunField::EnvList => match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.move_env_cursor(1),
                KeyCode::Up | KeyCode::Char('k') => self.move_env_cursor(-1),
                KeyCode::Delete | KeyCode::Backspace | KeyCode::Char('d') => {
                    self.remove_selected_env();
                }
                KeyCode::Enter | KeyCode::Char('e') => {
                    self.edit_selected_env();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.clear_all_env();
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    self.focus = RunField::EnvKey;
                }
                _ => {}
            },
            RunField::RunButton => {
                if key.code == KeyCode::Enter {
                    return Some(Action::Images(IA::RunContainer(self.build_run_request())));
                }
            }
            RunField::EnvFilePath => match key.code {
                KeyCode::Enter => self.load_env_from_path(),
                KeyCode::Backspace => {
                    self.env_file_path.pop();
                }
                KeyCode::Char(c) => {
                    self.env_file_path.push(c);
                }
                _ => {}
            },
            RunField::EnvKey => match key.code {
                KeyCode::Enter => {
                    if self.env_key_input.contains('=') || !self.env_val_input.is_empty() {
                        self.add_manual_env();
                    } else {
                        self.focus = RunField::EnvValue;
                    }
                }
                KeyCode::Backspace => {
                    self.env_key_input.pop();
                }
                KeyCode::Char(c) => {
                    self.env_key_input.push(c);
                }
                _ => {}
            },
            RunField::EnvValue => match key.code {
                KeyCode::Enter => self.add_manual_env(),
                KeyCode::Backspace => {
                    self.env_val_input.pop();
                }
                KeyCode::Char(c) => {
                    self.env_val_input.push(c);
                }
                _ => {}
            },
            RunField::ContainerName | RunField::Ports | RunField::Volumes => match key.code {
                KeyCode::Enter => {
                    self.focus = self.focus.next();
                }
                KeyCode::Backspace => {
                    if let Some(buf) = self.active_text_mut() {
                        buf.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(buf) = self.active_text_mut() {
                        buf.push(c);
                    }
                }
                _ => {}
            },
        }

        None
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.areas.name_area.contains(pos) {
                    self.focus = RunField::ContainerName;
                } else if self.areas.ports_area.contains(pos) {
                    self.focus = RunField::Ports;
                } else if self.areas.volumes_area.contains(pos) {
                    self.focus = RunField::Volumes;
                } else if self.areas.env_file_area.contains(pos) {
                    self.focus = RunField::EnvFilePath;
                } else if self.areas.load_env_btn.contains(pos) {
                    self.focus = RunField::EnvFilePath;
                    self.load_env_from_path();
                } else if self.areas.import_img_env_btn.contains(pos) {
                    self.import_image_default_env();
                } else if self.areas.env_key_area.contains(pos) {
                    self.focus = RunField::EnvKey;
                } else if self.areas.env_val_area.contains(pos) {
                    self.focus = RunField::EnvValue;
                } else if self.areas.add_env_btn.contains(pos) {
                    self.add_manual_env();
                } else if self.areas.clear_env_btn.contains(pos) {
                    self.clear_all_env();
                } else if self.areas.env_list_area.contains(pos) {
                    self.focus = RunField::EnvList;
                    let first_row_y = self.areas.env_list_area.y + 2;
                    if pos.y >= first_row_y {
                        let row_in_view = (pos.y - first_row_y) as usize;
                        let idx = self.env_offset + row_in_view;
                        if idx < self.env_vars.len() {
                            self.env_cursor = idx;
                        }
                    }
                } else if self.areas.run_btn.contains(pos) {
                    self.focus = RunField::RunButton;
                    return Some(Action::Images(IA::RunContainer(self.build_run_request())));
                } else if self.areas.cancel_btn.contains(pos) {
                    return Some(Action::Images(IA::BackToList));
                }
            }
            MouseEventKind::ScrollDown if self.areas.env_list_area.contains(pos) => {
                self.move_env_cursor(1);
            }
            MouseEventKind::ScrollUp if self.areas.env_list_area.contains(pos) => {
                self.move_env_cursor(-1);
            }
            _ => {}
        }
        None
    }

    fn update(&mut self, action: &Action) -> bool {
        let Action::Images(ia) = action else {
            return false;
        };

        match ia {
            IA::RunImageInspectLoaded(id, res) if *id == self.image_id => {
                if let Ok(inspect) = res {
                    self.apply_image_inspect(inspect);
                }
                true
            }
            IA::RunContainer(req) => {
                self.spawn_run(req.clone());
                true
            }
            IA::RunCompleted(Err(e)) => {
                self.loading = false;
                self.error_msg = Some(e.clone());
                true
            }
            _ => false,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.clamp_env_cursor();
        let props = RunViewProps {
            image_ref: &self.image_ref,
            exposed_ports_hint: &self.exposed_ports_hint,
            image_default_env_count: self.image_default_env.len(),
            focus: self.focus,
            container_name: &self.container_name,
            ports_input: &self.ports_input,
            volumes_input: &self.volumes_input,
            env_file_path: &self.env_file_path,
            env_key_input: &self.env_key_input,
            env_val_input: &self.env_val_input,
            env_vars: &self.env_vars,
            env_cursor: self.env_cursor,
            env_offset: self.env_offset,
            loading: self.loading,
            status_msg: self.status_msg.as_deref(),
            error_msg: self.error_msg.as_deref(),
        };

        ui::run_view::draw(frame, area, &props, &mut self.areas, &self.theme_color);
    }
}

