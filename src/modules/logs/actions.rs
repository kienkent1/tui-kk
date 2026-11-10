use serde::{Deserialize, Serialize};

use crate::modules::logs::log_dto::{LogEntry, LogFileSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogAction {
    Refresh,
    Loaded(u64, Result<Vec<LogFileSummary>, String>),
    OpenDetail(String),
    DetailLoaded(String, Result<Vec<LogEntry>, String>),
    DeleteFile(String),
    ClearFile(String),
    DeleteAll,
    ActionCompleted(Result<String, String>),
    FileDeleted(String),
    BackToList,

    SearchPush(char),
    SearchPop,
    SearchClear,
    CycleSort,
    ToggleOrder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogButtonAction {
    Remove,
}

