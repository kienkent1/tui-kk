use crate::tuikk_core::cli::RATE_RANGE;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
pub const DEFAULT_FRAME_RATE: f64 = 30.0;
pub const DEFAULT_TICK_RATE: f64 = 4.0;

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
#[serde(default)]
pub struct UiConfig {
    #[default(DEFAULT_FRAME_RATE)]
    pub frame_rate: f64,
    #[default(DEFAULT_TICK_RATE)]
    pub tick_rate: f64,
}

impl UiConfig {
    pub fn sanitized(mut self) -> Self {
        for (name, v, def) in [
            ("frame_rate", &mut self.frame_rate, DEFAULT_FRAME_RATE),
            ("tick_rate", &mut self.tick_rate, DEFAULT_TICK_RATE),
        ] {
            if !v.is_finite() {
                tracing::warn!("ui.{name} = {v} is not valid, change to {def}");
                *v = def;
            } else if !RATE_RANGE.contains(v) {
                let con = v.clamp(*RATE_RANGE.start(), *RATE_RANGE.end());
                tracing::warn!("ui.{name} = {v} outside the permitted range, change to {con}");
                *v = con;
            }
        }

        self
    }
}
