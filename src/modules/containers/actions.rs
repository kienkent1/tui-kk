use bollard::plugin::{ContainerInspectResponse, ContainerSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerAction {
    Refresh,
    Loaded(u64, Result<Vec<ContainerSummary>, String>),
    OpenDetail(String),
    DetailLoaded(String, Result<Box<ContainerInspectResponse>, String>),
    BackToList,

    SearchPush(char),
    SearchPop,
    SearchClear,
    CycleSort,
    ToggleOrder,
}
