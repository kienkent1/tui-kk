use crate::{
    modules::containers::{actions::ContainerAction as CA, container_service::ContainerService},
    shared::base::{
        base_action::Action,
        base_component::{Page, Tx},
        base_filter::BaseFilter,
        icons::Icons,
    },
    tuikk_core::{
        click_tracker::{ClickTracker, Clicks},
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};
use bollard::plugin::{ContainerSummary, ContainerSummaryStateEnum, PortSummaryTypeEnum};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState},
};
use ratatui_interact::components::{Button, ButtonState};
use std::{collections::BTreeSet, sync::Arc};
pub struct ContainerListPage {
    svc: Arc<ContainerService>,
    tx: Tx,
    is_local: bool,
    theme_color: ThemeColor,

    items: Vec<ContainerSummary>,
    filter: BaseFilter,
    clicks: ClickTracker,

    // search
    input_mode: bool,
    draft: String,
    prev_search: Option<String>,
    search_area: Rect,
    list_area: Rect,
    table_state: TableState,

    // button
    row_button_rects: Vec<((usize, Rect), (usize, Rect))>,
    hovered_row: Option<usize>,
    pressed_row: Option<usize>,

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
            clicks: ClickTracker::default(),
            input_mode: false,
            draft: String::new(),
            prev_search: None,
            search_area: Rect::default(),
            list_area: Rect::default(),
            table_state: TableState::default(),
            row_button_rects: Vec::new(),
            hovered_row: None,
            pressed_row: None,
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

    fn refresh(&mut self) {
        self.input_mode = false;
        self.draft = String::new();
        self.prev_search = None;
        self.filter.search = None;
        self.on_filter_change();
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
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Click on the search box.
                if self.search_area.contains(pos) {
                    if !self.input_mode {
                        return self.handle_command(Command::Search);
                    }
                    return None; // typing now, not resetting draft
                }

                // Click on the table
                if self.list_area.contains(pos) {
                    let click = self.clicks.listen(mouse.column, mouse.row);
                    if click == Clicks::Nothing {
                        return None;
                    }
                    // When clicking outside the table while entering data -> quit input mode
                    if self.input_mode {
                        self.input_mode = false;
                        if !self.is_local {
                            let q = self.draft.clone();
                            self.set_search(&q);
                            self.on_filter_change();
                            return None;
                        }
                    }

                    // 1 top border + 1 header = first data row
                    let first_row_y = self.list_area.y + 2;
                    if pos.y < first_row_y {
                        return None; // click on the borrder/header
                    }

                    let row_in_view = (pos.y - first_row_y) as usize;
                    if row_in_view >= self.viewport {
                        return None; // click on the bottom border
                    }

                    let idx = self.offset + row_in_view;
                    if idx >= self.rows().len() {
                        return None; // click on the empty space
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

            MouseEventKind::ScrollDown if self.list_area.contains(pos) => {
                self.move_cursor(1);
                return None;
            }
            MouseEventKind::ScrollUp if self.list_area.contains(pos) => {
                self.move_cursor(-1);
                return None;
            }
            _ => {}
        }

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

        self.search_area = search_area;
        self.list_area = table_area;
        // ── search box ──
        self.search_box(frame);

        // ── table ──
        self.draw_table(frame);
    }
}

// Draw fn
impl ContainerListPage {
    fn search_box(&mut self, frame: &mut Frame) {
        let (text, border) = if self.input_mode {
            (
                self.draft.clone(),
                Style::new().fg(self.theme_color.border_focus),
            )
        } else {
            (
                self.filter.search.clone().unwrap_or_default(),
                Style::new().fg(self.theme_color.border),
            )
        };

        let mode = if self.is_local { "local" } else { "remote" };

        let block = Block::bordered()
            .title(format!(" Search (/) · {mode} "))
            .border_style(border);
        let inner = block.inner(self.search_area);

        let text_color = Style::new().fg(self.theme_color.input_cursor);
        frame.render_widget(
            Paragraph::new(text).style(text_color).block(block),
            self.search_area,
        );

        if self.input_mode {
            let w = ratatui::text::Line::from(self.draft.as_str()).width() as u16;
            let x = (inner.x + w).min(inner.right().saturating_sub(1));
            frame.set_cursor_position((x, inner.y));
        }
    }

    fn draw_table(&mut self, frame: &mut Frame) {
        let [table_area, action_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(12)]).areas(self.list_area);
        self.viewport = table_area.height.saturating_sub(3) as usize; // 2 border + 1 header
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
                // Build a compact, deduplicated port string for the PORTS column.
                // Entries are keyed by (public, private, protocol); IPv4/IPv6 duplicates collapse.
                let ports: String = match &c.ports {
                    Some(list) => {
                        let unique: BTreeSet<(Option<u16>, u16, &str)> = list
                            .iter()
                            .map(|p| {
                                // Default to tcp when Docker omits the type
                                let proto = match p.typ {
                                    Some(PortSummaryTypeEnum::UDP) => "udp",
                                    Some(PortSummaryTypeEnum::SCTP) => "sctp",
                                    _ => "tcp",
                                };
                                (p.public_port, p.private_port, proto)
                            })
                            .collect();

                        unique
                            .into_iter()
                            .map(|(public, private, proto)| match public {
                                // Published: host:container/proto
                                Some(pub_p) => format!("{proto}/{pub_p}:{private}"),
                                // Exposed only: container/proto
                                None => format!("{proto}/{private}"),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                    None => String::new(),
                };
                Row::new(vec![
                    Cell::from(short_id.to_owned()),
                    Cell::from(name.to_owned()),
                    Cell::from(c.image.clone().unwrap_or_default()),
                    Cell::from(ports),
                    Cell::from(c.state.map(|s| s.to_string()).unwrap_or_default()),
                    Cell::from(c.status.clone().unwrap_or_default()),
                ])
            })
            .collect();

        let header = Row::new(["ID", "NAME", "IMAGE", "PORT", "STATE", "STATUS"]).style(
            Style::new()
                .fg(self.theme_color.list_header_text)
                .add_modifier(Modifier::BOLD),
        );

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
                Constraint::Length(13),     // ID   (12 chars + 1 padding)
                Constraint::Percentage(20), // NAME
                Constraint::Percentage(25), // IMAGE
                Constraint::Percentage(20), // PORTS
                Constraint::Length(10),     // STATE
                Constraint::Fill(1),        // STATUS
            ],
        )
        .header(header)
        .block(Block::bordered().fg(self.theme_color.border).title(title))
        .row_highlight_style(
            Style::new()
                .fg(self.theme_color.border)
                .bg(self.theme_color.text_selected)
                .add_modifier(Modifier::REVERSED),
        );

        let mut state = TableState::default();
        if len > 0 {
            state.select(Some(self.cursor - self.offset));
        }
        frame.render_stateful_widget(table, table_area, &mut state);

        // Action column: same top and bottom border so body rows line up with the table.
        let block = Block::bordered().border_style(Style::new().fg(self.theme_color.border));
        let inner = block.inner(action_area);
        frame.render_widget(block, action_area);

        // Header line of the action column (same height as the table header).
        frame.render_widget(
            Paragraph::new("Action").centered().style(
                Style::new()
                    .fg(self.theme_color.list_header_text)
                    .add_modifier(Modifier::BOLD),
            ),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );

        // Snapshot the running state of the visible rows so `rows` is not used past this point.
        // Index `i` matches the visible row order (same skip/take as `body`).
        let is_running: Vec<bool> = rows
            .iter()
            .skip(self.offset)
            .take(self.viewport)
            .map(|c| c.state == Some(ContainerSummaryStateEnum::RUNNING))
            .collect();
        // One button per visible row. Each button occupies exactly one terminal line.
        self.row_button_rects.clear();
        let body_top = inner.y + 1;
        for (i, idx) in (self.offset..len).take(self.viewport).enumerate() {
            let row_rect = Rect::new(inner.x, body_top + i as u16, inner.width, 1);
            let [state_rect, remove_rect] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                    .flex(Flex::SpaceEvenly)
                    .areas(row_rect);

            let (remove_style, state_style) = if self.pressed_row == Some(idx) {
                (
                    Style::new().add_modifier(Modifier::REVERSED),
                    Style::new().add_modifier(Modifier::REVERSED),
                )
            } else if self.hovered_row == Some(idx) {
                (
                    Style::new()
                        .fg(self.theme_color.border_focus)
                        .add_modifier(Modifier::BOLD),
                    Style::new()
                        .fg(self.theme_color.border_focus)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::new().fg(self.theme_color.error),
                    Style::new().fg(self.theme_color.text),
                )
            };

            // Running container -> show Stop, otherwise -> show Start
            let state_icon: &str = if is_running.get(i).copied().unwrap_or(false) {
                Icons::Stop.glyph()
            } else {
                Icons::Start.glyph()
            };

            frame.render_widget(
                Paragraph::new(state_icon).centered().style(state_style),
                state_rect,
            );
            frame.render_widget(
                Paragraph::new(Icons::Remove.glyph())
                    .centered()
                    .style(remove_style),
                remove_rect,
            );

            self.row_button_rects
                .push(((idx, state_rect), (idx, remove_rect)));
        }
    }
}
