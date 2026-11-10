use std::sync::Arc;

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
};

use crate::{
    modules::dashboard::{
        actions::DashboardAction,
        constants::{DASHBOARD_REFRESH_TICKS, MB},
        dashboard_dto::{DashboardSnapshot, MetricHistory},
        dashboard_service::DashboardService,
        ui,
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

pub struct DashboardPage {
    svc: Arc<DashboardService>,
    tx: Tx,
    theme_color: ThemeColor,

    snapshot: DashboardSnapshot,
    history: MetricHistory,

    loading: bool,
    error: Option<String>,
    req_seq: u64,
    tick_count: u64,

    cursor: usize,
    offset: usize,
    viewport: usize,
    bottom_area: Rect,
}

impl DashboardPage {
    pub fn new(services: &'static AppServices, tx: Tx, theme_color: &ThemeColor) -> Self {
        Self::with_service(services.dashboard.clone(), tx, theme_color)
    }

    pub fn with_service(svc: Arc<DashboardService>, tx: Tx, theme_color: &ThemeColor) -> Self {
        Self {
            svc,
            tx,
            theme_color: *theme_color,
            snapshot: DashboardSnapshot::default(),
            history: MetricHistory::default(),
            loading: false,
            error: None,
            req_seq: 0,
            tick_count: 0,
            cursor: 0,
            offset: 0,
            viewport: 8,
            bottom_area: Rect::default(),
        }
    }

    pub fn fetch(&mut self) {
        if self.loading {
            return;
        }
        self.req_seq += 1;
        let seq = self.req_seq;
        self.loading = true;

        let (svc, tx) = (self.svc.clone(), self.tx.clone());
        tokio::spawn(async move {
            let res = svc.fetch_snapshot().await.map_err(|e| e.to_string());
            let _ = tx.send(Action::Dashboard(DashboardAction::Loaded(seq, res)));
        });
    }

    pub fn clamp(&mut self) {
        let len = self.snapshot.containers.len();
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

    pub fn move_cursor(&mut self, delta: isize) {
        let len = self.snapshot.containers.len();
        if len == 0 {
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
        self.clamp();
    }
}

impl Page for DashboardPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Dashboard
    }

    fn captures_input(&self) -> bool {
        false
    }

    fn on_activate(&mut self) {
        self.fetch();
    }

    fn on_deactivate(&mut self) {}

    fn tick(&mut self) -> bool {
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.tick_count % DASHBOARD_REFRESH_TICKS == 0 {
            self.fetch();
        }
        true
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        match cmd {
            Command::Down => self.move_cursor(1),
            Command::Up => self.move_cursor(-1),
            Command::Refresh => {
                self.loading = false;
                self.fetch();
            }
            _ => {}
        }
        None
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::ScrollDown if self.bottom_area.contains(pos) => {
                self.move_cursor(1);
            }
            MouseEventKind::ScrollUp if self.bottom_area.contains(pos) => {
                self.move_cursor(-1);
            }
            _ => {}
        }
        None
    }

    fn update(&mut self, action: &Action) -> bool {
        match action {
            Action::Dashboard(DashboardAction::Refresh) => {
                self.loading = false;
                self.fetch();
                true
            }
            Action::Dashboard(DashboardAction::Loaded(seq, res)) => {
                if *seq != self.req_seq {
                    return false;
                }
                self.loading = false;
                match res {
                    Ok(snap) => {
                        let storage_mb = snap.storage.total_used() as f64 / MB;
                        self.history.push(
                            snap.total_cpu_percent,
                            snap.total_mem_percent,
                            storage_mb,
                        );
                        self.snapshot = snap.clone();
                        self.error = None;
                        self.clamp();
                    }
                    Err(e) => {
                        self.error = Some(e.clone());
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [top_area, charts_area, bottom_area] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Percentage(52),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [cpu_area, mem_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(charts_area);

        self.bottom_area = bottom_area;
        self.viewport = bottom_area.height.saturating_sub(3) as usize;
        self.clamp();

        ui::summary_cards::draw(
            frame,
            top_area,
            &self.snapshot,
            self.loading,
            self.error.as_deref(),
            &self.theme_color,
        );
        ui::cpu_chart::draw(
            frame,
            cpu_area,
            &self.snapshot,
            &self.history,
            &self.theme_color,
        );
        ui::mem_chart::draw(
            frame,
            mem_area,
            &self.snapshot,
            &self.history,
            &self.theme_color,
        );
        ui::storage_panel::draw(
            frame,
            bottom_area,
            &self.snapshot,
            self.cursor,
            self.offset,
            self.viewport,
            &self.theme_color,
        );
    }
}
