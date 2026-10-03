use crate::shared::base::base_action::Action;
use crate::tuikk_core::docker_conn::DockerConnection;
use crate::tuikk_core::route::Route;
use crate::tuikk_core::routers::Router;
use crate::tuikk_core::tui::{Event, Tui};
use color_eyre::Result;
use crossterm::event::{EventStream, KeyCode};
use crossterm::event::{KeyEvent, KeyModifiers};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::interval;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Quit,
    NextPage,
    PrevPage,
    Up,
    Down,
    Select,
    Back,
    Refresh,
    Stop,
    Restart,
    Logs,
}
pub struct App {
    router: Router,
    tx: UnboundedSender<Action>,
    rx: UnboundedReceiver<Action>,
    should_quit: bool,
}
impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            router: Router::new(),
            tx,
            rx,
            should_quit: false,
        }
    }

    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        tui.enter()?;
        self.router.activate_current(&self.tx);

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
                    self.router.tick();
                }
                Some(Event::Key(key)) => {
                    // Ctrl+C — global quit
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        self.should_quit = true;
                        continue;
                    }

                    // Tab / BackTab — top-level navigation
                    match key.code {
                        KeyCode::Tab => {
                            self.router.navigate(self.next_route(), &self.tx);
                        }
                        KeyCode::BackTab => {
                            self.router.navigate(self.prev_route(), &self.tx);
                        }
                        _ => {
                            if let Some(action) = self.router.handle_key(key) {
                                let _ = self.tx.send(action);
                            }
                        }
                    }
                }
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

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Navigate(route) => self.router.navigate(route, &self.tx),
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
}
