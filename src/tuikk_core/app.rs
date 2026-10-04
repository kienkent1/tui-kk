use crate::shared::base::base_action::Action;
use crate::tuikk_core::key_map::{Command, KeyMap};
use crate::tuikk_core::route::Route;
use crate::tuikk_core::routers::Router;
use crate::tuikk_core::tui::{Event, Tui};
use color_eyre::Result;
use crossterm::event::{EventStream, KeyCode, KeyEventKind};
use crossterm::event::{KeyEvent, KeyModifiers};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::interval;

const KEY_SEQ_TIMEOUT: Duration = Duration::from_secs(1);
pub struct App {
    router: Router,
    tx: UnboundedSender<Action>,
    rx: UnboundedReceiver<Action>,
    should_quit: bool,

    keymap: KeyMap,
    pending: Vec<KeyEvent>,
    pending_since: Option<Instant>,
}
impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            router: Router::new(&tx),
            tx,
            rx,
            should_quit: false,
            keymap: KeyMap::load(),
            pending: Vec::new(),
            pending_since: None,
        }
    }

    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        tui.enter()?;
        self.router.activate_current();

        while !self.should_quit {
            // Drain action channel first (none block)
            while let Ok(action) = self.rx.try_recv() {
                self.handle_action(action);
            }

            match tui.next_event().await {
                Some(Event::Render) => {
                    tui.draw(|f| self.router.draw(f, f.area()))?;
                }
                Some(Event::Tick) => {
                    if self
                        .pending_since
                        .is_some_and(|t| t.elapsed() > KEY_SEQ_TIMEOUT)
                    {
                        self.clear_pending();
                    }
                    self.router.tick();
                }
                Some(Event::Key(key)) => self.on_key(key),
                Some(Event::Resize(w, h)) => {
                    tui.draw(|f| self.router.draw(f, f.area()))?;
                }
                Some(Event::Error(e)) => {
                    tracing::error!(e)
                }
                _ => {}
            }
        }

        tui.exit()?;
        Ok(())
    }

    // ───────────────────────── actions / routes ─────────────────────────

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Navigate(route) => self.router.navigate(route),
            other => {
                self.router.dispatch(other);
            }
        }
    }

    fn next_route(&self) -> Route {
        use strum::EnumCount;
        let idx = (self.router.current_route() as usize + 1) % Route::COUNT;
        Route::from_repr(idx).unwrap_or_default()
    }

    fn prev_route(&self) -> Route {
        use strum::EnumCount;
        let idx = self.router.current_route() as usize;
        let prev = if idx == 0 { Route::COUNT - 1 } else { idx - 1 };
        Route::from_repr(prev).unwrap_or_default()
    }

    // ───────────────────────── key handling ─────────────────────────

    fn clear_pending(&mut self) {
        self.pending.clear();
        self.pending_since = None;
    }

    fn send(&self, action: Action) {
        let _ = self.tx.send(action);
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        let key = KeyEvent::new(key.code, key.modifiers);

        if self.router.captures_input() {
            self.clear_pending();
            if let Some(action) = self.router.handle_key(key) {
                self.send(action);
            }
            return;
        }

        self.pending.push(key);
        let scope = self.router.scope();

        match self.keymap.resolve(scope, &self.pending).copied() {
            Some(cmd) => {
                self.clear_pending();
                self.run_command(cmd);
            }

            None if self.keymap.has_prefix(scope, &self.pending) => {
                self.pending_since = Some(Instant::now());
            }
            None => {
                let was_chain = self.pending.len() > 1;
                self.clear_pending();
                if was_chain {
                    self.on_key(key);
                } else if let Some(action) = self.router.handle_key(key) {
                    self.send(action);
                }
            }
        }
    }

    fn run_command(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => self.should_quit = true,
            Command::NextPage => self.send(Action::Navigate(self.next_route())),
            Command::PrevPage => self.send(Action::Navigate(self.prev_route())),
            other => {
                if let Some(action) = self.router.handle_command(other) {
                    self.send(action);
                }
            }
        }
    }
}
