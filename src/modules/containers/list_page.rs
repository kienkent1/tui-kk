use std::sync::Arc;

use bollard::plugin::{ContainerSummary, ContainerSummaryStateEnum};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
};

use crate::{
    modules::containers::{
        actions::{ButtonAction, ContainerAction as CA},
        constants::ContainerSort,
        container_dto::{RowButton, RowTarget},
        container_service::ContainerService,
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

pub struct ContainerListPage {
    svc: Arc<ContainerService>,
    tx: Tx,
    is_local: bool,
    theme_color: ThemeColor,

    items: Vec<ContainerSummary>,
    filter: BaseFilter,
    sort_key: Option<ContainerSort>,
    sort_desc: bool,
    clicks: ClickTracker,

    // search
    input_mode: bool,
    draft: String,
    prev_search: Option<String>,
    search_area: Rect,
    list_area: Rect,

    // button
    row_buttons: Vec<RowButton>,
    hovered: Option<RowTarget>,
    pressed: Option<RowTarget>,

    // async
    loading: bool,
    error: Option<String>,
    req_seq: u64,

    // selection / scroll
    cursor: usize,
    offset: usize,
    viewport: usize,
    selected_id: Option<String>,
}

impl ContainerListPage {
    pub fn new(
        svc: Arc<ContainerService>,
        tx: Tx,
        is_local: bool,
        theme_color: &ThemeColor,
    ) -> Self {
        Self {
            svc,
            tx,
            is_local,
            theme_color: *theme_color,
            items: Vec::new(),
            filter: BaseFilter::default(),
            sort_key: None,
            sort_desc: false,
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
            selected_id: None,
        }
    }

    pub fn rows(&self) -> Vec<&ContainerSummary> {
        if self.is_local {
            self.svc.query_containers(&self.items, &self.filter)
        } else {
            self.items.iter().collect()
        }
    }

    pub fn row_id(&self, index: usize) -> Option<String> {
        self.rows().get(index).and_then(|c| c.id.clone())
    }

    pub fn find_by_id(&self, id: &str) -> Option<ContainerSummary> {
        self.items
            .iter()
            .find(|c| c.id.as_deref() == Some(id))
            .cloned()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn set_viewport(&mut self, viewport: usize) {
        self.viewport = viewport.max(1);
    }

    pub fn set_items(&mut self, items: Vec<ContainerSummary>) {
        self.items = items;
        self.sync();
    }

    fn fetch(&mut self) {
        self.req_seq += 1;
        let seq = self.req_seq;
        self.loading = true;
        self.error = None;

        let (svc, tx, filter, is_local) = (
            self.svc.clone(),
            self.tx.clone(),
            self.filter.clone(),
            self.is_local,
        );

        tokio::spawn(async move {
            let res = if is_local {
                svc.fetch_containers(&BaseFilter::default(), Some(true), None)
                    .await
            } else {
                svc.get_containers(filter, Some(true)).await
            }
            .map_err(|e| e.to_string());

            let _ = tx.send(Action::Containers(CA::Loaded(seq, res)));
        });
    }

    fn spawn_toggle_state(&mut self, id: String, should_start: bool) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = if should_start {
                svc.start_container(&id).await
            } else {
                svc.stop_container(&id).await
            }
            .map(|_| id)
            .map_err(|e| e.to_string());
            let _ = tx.send(Action::Containers(CA::ActionCompleted(res)));
        });
    }

    fn spawn_restart(&mut self, id: String) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = svc
                .restart_container(&id)
                .await
                .map(|_| id)
                .map_err(|e| e.to_string());
            let _ = tx.send(Action::Containers(CA::ActionCompleted(res)));
        });
    }

    fn spawn_remove(&mut self, id: String) {
        self.loading = true;
        self.error = None;
        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = svc
                .remove_container(&id, true)
                .await
                .map(|_| id)
                .map_err(|e| e.to_string());
            let _ = tx.send(Action::Containers(CA::ActionCompleted(res)));
        });
    }

    fn on_filter_change(&mut self) {
        self.reset_selection();
        if self.is_local {
            self.sync();
        } else {
            self.fetch();
        }
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
            None => ContainerSort::Name,
        };
        self.sort_key = Some(next_key);
        self.filter.sort = Some(next_key.to_filter_string(self.sort_desc));
        self.on_filter_change();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_desc = !self.sort_desc;
        let key = self.sort_key.unwrap_or(ContainerSort::Name);
        self.sort_key = Some(key);
        self.filter.sort = Some(key.to_filter_string(self.sort_desc));
        self.on_filter_change();
    }

    // ───────────── selection ─────────────

    fn reset_selection(&mut self) {
        self.cursor = 0;
        self.offset = 0;
        self.selected_id = None;
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
                .selected_id
                .as_deref()
                .and_then(|id| rows.iter().position(|c| c.id.as_deref() == Some(id)));

            (rows.len(), found)
        };

        if let Some(index) = found {
            self.cursor = index;
        } else {
            self.cursor = 0;
            self.selected_id = self.row_id(0);
        }

        self.clamp(len);
        self.selected_id = self.row_id(self.cursor);
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }

        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.selected_id = self.row_id(self.cursor);
        self.clamp(len);
    }

    fn refresh(&mut self) {
        self.input_mode = false;
        self.draft = String::new();
        self.prev_search = None;
        self.filter.search = None;
        self.fetch();
    }

    fn click_button_event(&self, pos: &Position) -> Option<RowTarget> {
        self.row_buttons
            .iter()
            .find(|b| b.rect.contains(*pos))
            .map(|b| b.target)
    }

    fn handle_button_event(&mut self, target: RowTarget) -> Option<Action> {
        let rows = self.rows();
        let c = rows.get(target.row)?;
        let id = c.id.clone()?;
        let running = c.state == Some(ContainerSummaryStateEnum::RUNNING);

        match target.action {
            ButtonAction::ToggleState if running => {
                Some(Action::Containers(CA::ToggleState(id, false)))
            }
            ButtonAction::ToggleState => Some(Action::Containers(CA::ToggleState(id, true))),
            ButtonAction::Remove => Some(Action::Containers(CA::Remove(id))),
        }
    }
}

impl Page for ContainerListPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Containers
    }

    fn captures_input(&self) -> bool {
        self.input_mode
    }

    fn on_activate(&mut self) {
        if self.items.is_empty() && !self.loading {
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
                let id = self.row_id(self.cursor)?;
                return Some(Action::Containers(CA::OpenDetail(id)));
            }
            Command::Stop => {
                let rows = self.rows();
                let c = rows.get(self.cursor)?;
                let id = c.id.clone()?;
                let running = c.state == Some(ContainerSummaryStateEnum::RUNNING);
                return Some(Action::Containers(CA::ToggleState(id, !running)));
            }
            Command::Delete => {
                let id = self.row_id(self.cursor)?;
                return Some(Action::Containers(CA::Remove(id)));
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

        if self.is_local {
            let q = self.draft.clone();
            self.set_search(&q);
            self.on_filter_change();
        }
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
                _ => {}
            }
            return None;
        }

        match key.code {
            KeyCode::Enter => {
                self.input_mode = false;
                if !self.is_local {
                    let q = self.draft.clone();
                    self.set_search(&q);
                    self.on_filter_change();
                }
            }
            KeyCode::Esc => {
                self.input_mode = false;
                if self.is_local {
                    self.filter.search = self.prev_search.take();
                    self.on_filter_change();
                }
            }
            KeyCode::Backspace => {
                self.draft.pop();
                if self.is_local {
                    let q = self.draft.clone();
                    self.set_search(&q);
                    self.on_filter_change();
                }
            }
            KeyCode::Char(c) => {
                self.draft.push(c);
                if self.is_local {
                    let q = self.draft.clone();
                    self.set_search(&q);
                    self.on_filter_change();
                }
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
                    self.selected_id = self.row_id(target.row);
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
                        if !self.is_local {
                            let q = self.draft.clone();
                            self.set_search(&q);
                            self.on_filter_change();
                            return None;
                        }
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
                    self.selected_id = self.row_id(idx);
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
        let Action::Containers(ca) = action else {
            return false;
        };

        match ca {
            CA::Refresh => {
                self.refresh();
                true
            }
            CA::Loaded(seq, res) => {
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
            CA::ToggleState(id, should_start) => {
                self.spawn_toggle_state(id.clone(), *should_start);
                true
            }
            CA::Restart(id) => {
                self.spawn_restart(id.clone());
                true
            }
            CA::Remove(id) => {
                self.spawn_remove(id.clone());
                true
            }
            CA::ActionCompleted(res) => {
                self.loading = false;
                match res {
                    Ok(_) => self.fetch(),
                    Err(e) => self.error = Some(e.clone()),
                }
                true
            }
            CA::CycleSort => {
                self.cycle_sort();
                true
            }
            CA::ToggleOrder => {
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
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(12)]).areas(list_area);

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
            self.is_local,
            &self.theme_color,
        );

        let is_running: Vec<bool> = {
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
            rows.iter()
                .skip(self.offset)
                .take(self.viewport)
                .map(|c| c.state == Some(ContainerSummaryStateEnum::RUNNING))
                .collect()
        };

        ui::action_column::draw(
            frame,
            action_area,
            len,
            self.offset,
            self.viewport,
            &is_running,
            self.hovered,
            self.pressed,
            &mut self.row_buttons,
            &self.theme_color,
        );
    }
}
