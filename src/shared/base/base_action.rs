use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Display, Serialize, Deserialize)]
pub enum Action {
    //Only in runtime
    #[serde(skip)] Tick,
    #[serde(skip)] Render,
    #[serde(skip)] Resize(u16, u16),
    #[serde(skip)] Resume,
    #[serde(skip)] Error(String),

    //global: can bind
    Suspend,
    Quit,
    ClearScreen,
    Help,
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
