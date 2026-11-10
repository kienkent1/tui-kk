use serde::{Deserialize, Serialize};

use crate::modules::dashboard::dashboard_dto::DashboardSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardAction {
    Refresh,
    Loaded(u64, Result<DashboardSnapshot, String>),
}
