use bollard::plugin::{ContainerInspectResponse, ContainerSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerAction {
    Refresh,
    Loaded(u64, Result<Vec<ContainerSummary>, String>),
    OpenDetail(String),
    DetailLoaded(String, Result<ContainerInspectResponse, String>),
    ToggleState(String, bool),
    Restart(String),
    Remove(String),
    ActionCompleted(Result<String, String>),
    BackToList,

    SearchPush(char),
    SearchPop,
    SearchClear,
    CycleSort,
    ToggleOrder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonAction {
    ToggleState,
    Remove,
}
