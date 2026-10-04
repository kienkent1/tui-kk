use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Maximum interval between two consecutive clicks in the same sequence
const MULTI_CLICK_GAP: Duration = Duration::from_millis(400);
/// Maximum total duration of the entire sequence, calculated from the first click.
const MULTI_CLICK_TOTAL: Duration = Duration::from_millis(900);
/// Ignore clicks that occur too close together (mouse noise, repeated events).
const MIN_CLICK_GAP: Duration = Duration::from_millis(30);
pub const MAX_COUNT: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Clicks {
    #[default]
    Nothing,
    Click,
    DoubleClick,
    TripleClick,
    FourClicks,
}

impl Clicks {
    pub fn from_count(n: u8) -> Self {
        match n {
            1 => Clicks::Click,
            2 => Clicks::DoubleClick,
            3 => Clicks::TripleClick,
            4 => Clicks::FourClicks,
            _ => Clicks::Nothing,
        }
    }
}

#[derive(Default)]
pub struct ClickTracker {
    first: Option<Instant>,
    last: Option<(Instant, u16, u16)>,
    count: u8,
    click: Clicks,
}

impl ClickTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    /// Enum form of the last accepted click.
    pub fn click(&self) -> Clicks {
        self.click
    }

    /// `key` identifies the target (row index, button id, ...).
    /// Returns `Nothing` if the event was ignored as noise.
    pub fn listen(&mut self, col: u16, row: u16) -> Clicks {
        self.listen_at(Instant::now(), col, row)
    }

    pub fn listen_at(&mut self, now: Instant, col: u16, row: u16) -> Clicks {
        if let Some((t, _, _)) = self.last {
            if now.duration_since(t) < MIN_CLICK_GAP {
                return Clicks::Nothing; // nhiễu, state không đổi
            }
        }

        let continues = match (self.first, self.last) {
            (Some(f), Some((t, c, r))) => {
                self.count < MAX_COUNT
                    && c == col
                    && r == row
                    && now.duration_since(t) <= MULTI_CLICK_GAP
                    && now.duration_since(f) <= MULTI_CLICK_TOTAL
            }
            _ => false,
        };

        if continues {
            self.count += 1;
        } else {
            self.count = 1;
            self.first = Some(now);
        }

        self.last = Some((now, col, row));
        self.click = Clicks::from_count(self.count);
        self.click
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }
}
