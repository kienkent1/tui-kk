use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
};

use crate::{
    modules::logs::{
        actions::{LogAction as LA, LogButtonAction},
        constants::LogSort,
        log_dto::{LogFileSummary, LogRowButton, LogRowTarget},
        log_service::LogService,
        ui,
    },
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
        base_filter::BaseFilter,
    },
    tuikk_core::{
        click_tracker::{ClickTracker, Clicks},
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

pub struct LogListPage {
    svc: Arc<LogService>,
    tx: Tx,
    theme_color: ThemeColor,

    items: Vec<LogFileSummary>,
    filter: BaseFilter,
    sort_key: Option<LogSort>,
    sort_desc: bool,
    clicks: ClickTracker,

    // search
    input_mode: bool,
    draft: String,
    prev_search: Option<String>,
    search_area: Rect,
    list_area: Rect,

    // button
    row_buttons: Vec<LogRowButton>,
    hovered: Option<LogRowTarget>,
    pressed: Option<LogRowTarget>,

    // async
    loading: bool,
    error: Option<String>,
    req_seq: u64,

    // selection / scroll
    cursor: usize,
    offset: usize,
    viewport: usize,
    selected_file: Option<String>,
}

impl LogListPage {
    pub fn new(svc: Arc<LogService>, tx: Tx, theme_color: &ThemeColor) -> Self {
        Self {
            svc,
            tx,
            theme_color: *theme_color,
            items: Vec::new(),
            filter: BaseFilter::default(),
            sort_key: None,
            sort_desc: true,
            clicks: ClickTracker::default(),
            input_mode: false,
            draft: String::new(),
            prev_search: None,
            search_area: Rect::default(),
            list_area: Rect::default(),
            row_buttons: Vec::new(),
            hovered: None,
            pressed: None,
            loading: false,
            error: None,
            req_seq: 0,
            cursor: 0,
            offset: 0,
            viewport: 20,
            selected_file: None,
        }
    }

    pub fn rows(&self) -> Vec<&LogFileSummary> {
        self.svc.query_log_files(&self.items, &self.filter)
    }

    pub fn row_file_name(&self, index: usize) -> Option<String> {
        self.rows().get(index).map(|f| f.file_name.clone())
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn selected_file(&self) -> Option<&str> {
        self.selected_file.as_deref()
    }

    pub fn set_viewport(&mut self, viewport: usize) {
        self.viewport = viewport.max(1);
    }

    pub fn set_items(&mut self, items: Vec<LogFileSummary>) {
        self.items = items;
        self.sync();
    }

    pub fn fetch(&mut self) {
        self.req_seq += 1;
        let seq = self.req_seq;
        self.loading = true;
        self.error = None;

        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::task::spawn_blocking(move || {
            let res = svc.fetch_log_files().map_err(|e| e.to_string());
            let _ = tx.send(Action::LogsPage(LA::Loaded(seq, res)));
        });
    }

    fn spawn_delete_file(&mut self, file_name: String) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::task::spawn_blocking(move || {
            let res = svc
                .delete_log_file(&file_name)
                .map(|_| file_name.clone())
                .map_err(|e| e.to_string());
            if res.is_ok() {
                let _ = tx.send(Action::LogsPage(LA::FileDeleted(file_name)));
            }
            let _ = tx.send(Action::LogsPage(LA::ActionCompleted(res)));
        });
    }

    fn spawn_clear_file(&mut self, file_name: String) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::task::spawn_blocking(move || {
            let res = svc
                .clear_log_file(&file_name)
                .map(|_| file_name)
                .map_err(|e| e.to_string());
            let _ = tx.send(Action::LogsPage(LA::ActionCompleted(res)));
        });
    }

    fn spawn_delete_all(&mut self) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::task::spawn_blocking(move || {
            let res = svc
                .delete_all_logs()
                .map(|cnt| format!("deleted {cnt} log files"))
                .map_err(|e| e.to_string());
            let _ = tx.send(Action::LogsPage(LA::ActionCompleted(res)));
        });
    }

    fn on_filter_change(&mut self) {
        self.reset_selection();
        self.sync();
    }

    pub fn set_search(&mut self, s: &str) {
        let s = s.trim();
        self.filter.search = if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        };
    }

    pub fn cycle_sort(&mut self) {
        let next_key = match self.sort_key {
            Some(k) => k.next(),
            None => LogSort::Errors,
        };
        self.sort_key = Some(next_key);
        self.filter.sort = Some(next_key.to_filter_string(self.sort_desc));
        self.on_filter_change();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_desc = !self.sort_desc;
        let key = self.sort_key.unwrap_or(LogSort::Modified);
        self.sort_key = Some(key);
        self.filter.sort = Some(key.to_filter_string(self.sort_desc));
        self.on_filter_change();
    }

    // ───────────── selection ─────────────

    fn reset_selection(&mut self) {
        self.cursor = 0;
        self.offset = 0;
        self.selected_file = None;
    }

    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.cursor = 0;
            self.offset = 0;
            return;
        }

        let h = self.viewport.max(1);
        self.cursor = self.cursor.min(len - 1);

        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + h {
            self.offset = self.cursor - h + 1;
        }
        self.offset = self.offset.min(len.saturating_sub(h));
    }

    pub fn sync(&mut self) {
        let (len, found) = {
            let rows = self.rows();
            let found = self
                .selected_file
                .as_deref()
                .and_then(|name| rows.iter().position(|f| f.file_name == name));

            (rows.len(), found)
        };

        if let Some(index) = found {
            self.cursor = index;
        } else {
            self.cursor = 0;
            self.selected_file = self.row_file_name(0);
        }

        self.clamp(len);
        self.selected_file = self.row_file_name(self.cursor);
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }

        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.selected_file = self.row_file_name(self.cursor);
        self.clamp(len);
    }

    fn refresh(&mut self) {
        self.input_mode = false;
        self.draft = String::new();
        self.prev_search = None;
        self.filter.search = None;
        self.fetch();
    }

    fn click_button_event(&self, pos: &Position) -> Option<LogRowTarget> {
        self.row_buttons
            .iter()
            .find(|b| b.rect.contains(*pos))
            .map(|b| b.target)
    }

    fn handle_button_event(&mut self, target: LogRowTarget) -> Option<Action> {
        let file_name = self.row_file_name(target.row)?;
        match target.action {
            LogButtonAction::Remove => Some(Action::LogsPage(LA::DeleteFile(file_name))),
        }
    }
}

impl Page for LogListPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Logs
    }

    fn captures_input(&self) -> bool {
        self.input_mode
    }

    fn on_activate(&mut self) {
        if !self.loading {
            self.fetch();
        }
    }

    fn on_deactivate(&mut self) {}

    fn tick(&mut self) -> bool {
        true
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        match cmd {
            Command::Down => self.move_cursor(1),
            Command::Up => self.move_cursor(-1),
            Command::Refresh => self.refresh(),
            Command::Search => {
                self.prev_search = self.filter.search.clone();
                self.draft = self.filter.search.clone().unwrap_or_default();
                self.input_mode = true;
            }
            Command::Select => {
                let name = self.row_file_name(self.cursor)?;
                return Some(Action::LogsPage(LA::OpenDetail(name)));
            }
            Command::Delete => {
                let name = self.row_file_name(self.cursor)?;
                return Some(Action::LogsPage(LA::DeleteFile(name)));
            }
            Command::Back => {
                if self.filter.search.is_some() {
                    self.filter.search = None;
                    self.draft.clear();
                    self.on_filter_change();
                }
            }
            _ => {}
        }

        None
    }

    fn handle_paste(&mut self, text: &str) -> Option<Action> {
        if !self.input_mode {
            return None;
        }

        self.draft.extend(text.chars().filter(|c| !c.is_control()));
        let q = self.draft.clone();
        self.set_search(&q);
        self.on_filter_change();
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if !self.input_mode {
            match key.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.cycle_sort();
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    self.toggle_sort_order();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    let name = self.row_file_name(self.cursor)?;
                    return Some(Action::LogsPage(LA::ClearFile(name)));
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    return Some(Action::LogsPage(LA::DeleteAll));
                }
                _ => {}
            }
            return None;
        }

        match key.code {
            KeyCode::Enter => {
                self.input_mode = false;
            }
            KeyCode::Esc => {
                self.input_mode = false;
                self.filter.search = self.prev_search.take();
                self.on_filter_change();
            }
            KeyCode::Backspace => {
                self.draft.pop();
                let q = self.draft.clone();
                self.set_search(&q);
                self.on_filter_change();
            }
            KeyCode::Char(c) => {
                self.draft.push(c);
                let q = self.draft.clone();
                self.set_search(&q);
                self.on_filter_change();
            }
            _ => {}
        }
        None
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let pos = Position::new(mouse.column, mouse.row);
        let button_clicked = self.click_button_event(&pos);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(target) = button_clicked {
                    self.pressed = Some(target);
                    self.cursor = target.row;
                    self.selected_file = self.row_file_name(target.row);
                    return None;
                }
                self.pressed = None;

                if self.search_area.contains(pos) {
                    if !self.input_mode {
                        return self.handle_command(Command::Search);
                    }
                    return None;
                }

                if self.list_area.contains(pos) {
                    let click = self.clicks.listen(mouse.column, mouse.row);
                    if click == Clicks::Nothing {
                        return None;
                    }
                    if self.input_mode {
                        self.input_mode = false;
                    }

                    let first_row_y = self.list_area.y + 2;
                    if pos.y < first_row_y {
                        return None;
                    }

                    let row_in_view = (pos.y - first_row_y) as usize;
                    if row_in_view >= self.viewport {
                        return None;
                    }

                    let idx = self.offset + row_in_view;
                    if idx >= self.rows().len() {
                        return None;
                    }

                    self.cursor = idx;
                    self.selected_file = self.row_file_name(idx);
                    if click == Clicks::DoubleClick {
                        self.clicks.reset();
                        return self.handle_command(Command::Select);
                    }
                }

                return None;
            }

            MouseEventKind::Up(MouseButton::Left) => {
                let pressed = self.pressed.take();
                if let (Some(p), Some(h)) = (pressed, button_clicked) {
                    if p == h {
                        return self.handle_button_event(h);
                    }
                }
            }

            MouseEventKind::ScrollDown if self.list_area.contains(pos) => {
                self.move_cursor(1);
                return None;
            }
            MouseEventKind::ScrollUp if self.list_area.contains(pos) => {
                self.move_cursor(-1);
                return None;
            }
            MouseEventKind::Moved => self.hovered = button_clicked,
            _ => {}
        }

        None
    }

    fn update(&mut self, action: &Action) -> bool {
        let Action::LogsPage(la) = action else {
            return false;
        };

        match la {
            LA::Refresh => {
                self.refresh();
                true
            }
            LA::Loaded(seq, res) => {
                if *seq != self.req_seq {
                    return false;
                }
                self.loading = false;
                match res {
                    Ok(items) => {
                        self.items = items.clone();
                        self.error = None;
                        self.sync();
                    }
                    Err(e) => self.error = Some(e.clone()),
                }
                true
            }
            LA::DeleteFile(file_name) => {
                self.spawn_delete_file(file_name.clone());
                true
            }
            LA::ClearFile(file_name) => {
                self.spawn_clear_file(file_name.clone());
                true
            }
            LA::DeleteAll => {
                self.spawn_delete_all();
                true
            }
            LA::ActionCompleted(res) => {
                self.loading = false;
                match res {
                    Ok(_) => self.fetch(),
                    Err(e) => self.error = Some(e.clone()),
                }
                true
            }
            LA::CycleSort => {
                self.cycle_sort();
                true
            }
            LA::ToggleOrder => {
                self.toggle_sort_order();
                true
            }
            _ => false,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [search_area, list_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(area);
        let [table_area, action_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(10)]).areas(list_area);

        self.search_area = search_area;
        self.list_area = list_area;
        self.viewport = table_area.height.saturating_sub(3) as usize;

        let len = self.rows().len();
        self.clamp(len);

        ui::search_box::draw(
            frame,
            self.search_area,
            self.input_mode,
            &self.draft,
            self.filter.search.as_deref(),
            self.filter.sort.as_deref(),
            self.svc.log_dir(),
            &self.theme_color,
        );

        {
            let rows = self.rows();
            ui::table::draw(
                frame,
                table_area,
                &rows,
                self.offset,
                self.viewport,
                self.cursor,
                self.loading,
                self.error.as_deref(),
                &self.theme_color,
            );
        }

        ui::action_column::draw(
            frame,
            action_area,
            len,
            self.offset,
            self.viewport,
            self.hovered,
            self.pressed,
            &mut self.row_buttons,
            &self.theme_color,
        );
    }
}
