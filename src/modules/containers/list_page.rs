use std::sync::Arc;

use crate::{
    modules::containers::{actions::ContainerAction as CA, container_service::ContainerService},
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
        base_filter::BaseFilter,
    },
    tuikk_core::key_map::{Command, KeyScope},
};
use bollard::plugin::ContainerSummary;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Rows},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState},
};
pub struct ContainerListPage {
    svc: Arc<ContainerService>,
    tx: Tx,
    is_local: bool,

    items: Vec<ContainerSummary>,
    filter: BaseFilter,

    // search
    input_mode: bool,
    draft: String,
    prev_search: Option<String>,

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
    pub fn new(svc: Arc<ContainerService>, tx: Tx, is_local: bool) -> Self {
        Self {
            svc,
            tx,
            is_local,
            items: Vec::new(),
            filter: BaseFilter::default(),
            input_mode: false,
            draft: String::new(),
            prev_search: None,
            loading: false,
            error: None,
            req_seq: 0,
            cursor: 0,
            offset: 0,
            viewport: 20,
            selected_id: None,
        }
    }

    fn rows(&self) -> Vec<&ContainerSummary> {
        if self.is_local {
            self.svc.query_containers(&self.items, &self.filter)
        } else {
            self.items.iter().collect()
        }
    }

    fn row_id(&self, index: usize) -> Option<String> {
        self.rows().get(index).and_then(|c| c.id.clone())
    }

    pub fn find_by_id(&self, id: &str) -> Option<ContainerSummary> {
        self.items
            .iter()
            .find(|c| c.id.as_deref() == Some(id))
            .cloned()
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

    fn on_filter_change(&mut self) {
        self.reset_selection();
        if self.is_local {
            self.sync();
        } else {
            self.fetch();
        }
    }

    fn set_search(&mut self, s: &str) {
        let s = s.trim();
        self.filter.search = if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        }
    }

    // ───────────── selection ─────────────

    fn reset_selection(&mut self) {
        self.cursor = 0;
        self.offset = 0;
        self.selected_id = None;
    }

    fn clamp(&mut self, len: usize) {
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

    fn move_cursor(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }

        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.selected_id = self.row_id(self.cursor);
        self.clamp(len);
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
            Command::Refresh => self.fetch(),
            Command::Search => {
                self.prev_search = self.filter.search.clone();
                self.draft = self.filter.search.clone().unwrap_or_default();
                self.input_mode = true;
            }
            Command::Select => {
                let id = self.row_id(self.cursor)?;
                return Some(Action::Containers(CA::OpenDetail(id)));
            }
            Command::Back => {
                if self.filter.search.is_none() {
                    self.filter.search = None;
                    self.draft.clear();
                    self.on_filter_change();
                }
            }
            _ => {}
        }

        None
    }

    /// Only called in input mode (or when the key is unassigned).
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if !self.input_mode {
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
        None
    }
    fn update(&mut self, action: &Action) -> bool {
        let Action::Containers(CA::Loaded(seq, res)) = action else {
            return false;
        };

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
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [search_area, table_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(area);

        // ── search box ──

        let (text, border) = if self.input_mode {
            (format!("{}|", self.draft), Style::new().fg(Color::Yellow))
        } else {
            (
                self.filter.search.clone().unwrap_or_default(),
                Style::new().fg(Color::DarkGray),
            )
        };

        let mode = if self.is_local { "local" } else { "remote" };
        frame.render_widget(
            Paragraph::new(text).block(
                Block::bordered()
                    .title(format!(" Search (/) · {mode} "))
                    .border_style(border),
            ),
            search_area,
        );

        // ── table ──
        self.viewport = table_area.height.saturating_sub(3) as usize; // 2 viền + 1 header
        let len = self.rows().len();
        self.clamp(len);

        let rows = self.rows();
        let body: Vec<Row> = rows
            .iter()
            .skip(self.offset)
            .take(self.viewport)
            .map(|c| {
                let name = c
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.trim_start_matches('/'))
                    .unwrap_or("-");
                let short_id =
                    c.id.as_deref()
                        .map(|s| &s[..s.len().min(12)])
                        .unwrap_or("-");
                Row::new(vec![
                    Cell::from(name.to_owned()),
                    Cell::from(c.image.clone().unwrap_or_default()),
                    Cell::from(c.state.map(|s| s.to_string()).unwrap_or_default()),
                    Cell::from(c.status.clone().unwrap_or_default()),
                    Cell::from(short_id.to_owned()),
                ])
            })
            .collect();

        let header = Row::new(["NAME", "IMAGE", "STATE", "STATUS", "ID"])
            .style(Style::new().add_modifier(Modifier::BOLD));

        let mut title = format!(" Containers ({len}) ");
        if self.loading {
            title.push_str("· loading… ");
        }
        if let Some(e) = &self.error {
            title.push_str(&format!("· error: {e} "));
        }

        let table = Table::new(
            body,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Length(10),
                Constraint::Percentage(25),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(Block::bordered().title(title))
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        let mut state = TableState::default();
        if len > 0 {
            state.select(Some(self.cursor - self.offset));
        }
        frame.render_stateful_widget(table, table_area, &mut state);
    }
}
