use crate::tuikk_core::docker_conn::DockerConnection;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use crossterm::event::{Event, EventStream, KeyCode};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
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
pub struct App;
impl App {
    pub async fn run(terminal: &mut DefaultTerminal) -> Result<()> {
        let mut event_stream = EventStream::new();

        let mut tick_timer = interval(Duration::from_secs(2));

        let docker = DockerConnection::global()?;

        let mut container_count = 0;
        loop {
            terminal.draw(|f| render(f, container_count))?;

            tokio::select! {
                maybe_event = event_stream.next() => {
                    if let Some(Ok(Event::Key(key))) = maybe_event {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            break;
                        }
                    }
                }

                _ = tick_timer.tick() => {
                    if let Ok(containers) = docker.list_containers(None).await {
                        container_count = containers.len();
                    }
                }
            }
        }

        Ok(())
    }

    fn render(frame: &mut Frame, container_count: usize) {
        let text = format!("TUI-KK | Running Containers: {container_count} (Press 'q' to quit)");
        frame.render_widget(text, frame.area());
    }
}
