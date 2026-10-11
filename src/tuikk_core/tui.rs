use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use color_eyre::Result;
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CtEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent,
    },
    execute,
};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::time::{Interval, MissedTickBehavior, interval};

use crate::shared::helpers::ext_log::ResultExt;
#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
    Error(String),
}

pub struct Tui {
    terminal: DefaultTerminal,
    events: EventStream,
    tick: Interval,
    render: Interval,
    mouse: bool,
    paste: bool,
    active: bool,
}

fn make_interval(rate: f64) -> Interval {
    let mut i = interval(Duration::from_secs_f64(1.0 / rate));
    i.set_missed_tick_behavior(MissedTickBehavior::Skip);
    i
}

impl Tui {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Self {
        Self {
            terminal: ratatui::init(),
            events: EventStream::new(),
            tick: make_interval(tick_rate),
            render: make_interval(frame_rate),
            mouse: false,
            paste: false,
            active: false,
        }
    }

    pub fn mouse(mut self, on: bool) -> Self {
        self.mouse = on;
        self
    }
    pub fn paste(mut self, on: bool) -> Self {
        self.paste = on;
        self
    }

    pub fn enter(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }
        self.terminal = ratatui::init();
        let mut out = std::io::stdout();
        if self.mouse {
            execute!(out, EnableMouseCapture).log_warn()?;
        }
        if self.paste {
            execute!(out, EnableBracketedPaste).log_warn()?;
        }
        self.active = true;
        Ok(())
    }

    pub fn exit(&mut self) -> color_eyre::Result<()> {
        if !self.active {
            return Ok(());
        }

        let mut out = std::io::stdout();
        if self.mouse {
            execute!(out, DisableMouseCapture).log_warn()?;
        }
        if self.paste {
            execute!(out, DisableBracketedPaste).log_warn()?;
        }
        ratatui::restore();
        self.active = false;
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(unix)]
        let raise_result = signal_hook::low_level::raise(signal_hook::consts::SIGTSTP);
        self.enter()?;
        self.terminal.clear()?;

        #[cfg(unix)]
        raise_result?;
        Ok(())
    }

    pub fn restart(&mut self) -> Result<()> {
        self.exit()?;
        self.enter()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn toggle_mouse_capture(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        self.mouse = !self.mouse;
        let mut out = std::io::stdout();
        if self.mouse {
            execute!(out, EnableMouseCapture).log_warn()?;
        } else {
            execute!(out, DisableMouseCapture).log_warn()?;
        }

        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        loop {
            let ev = tokio::select! {
                _ = self.tick.tick() => Event::Tick,
                _ = self.render.tick() => Event::Render,
                maybe = self.events.next() => match  maybe?  {
                    Ok(CtEvent::Key(k)) if k.kind == KeyEventKind::Press => Event::Key(k),
                    Ok(CtEvent::Mouse(m)) => Event::Mouse(m),
                    Ok(CtEvent::Resize(w, h)) => Event::Resize(w, h),
                    Ok(CtEvent::Paste(s)) => Event::Paste(s),
                    Ok(_) => continue,
                    Err(e) => Event::Error(e.to_string()),
            }
            };

            return Some(ev);
        }
    }
}

impl Deref for Tui {
    type Target = DefaultTerminal;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}
