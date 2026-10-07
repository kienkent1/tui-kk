use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, EnumCount, FromRepr, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    //#[default]
    Dashboard,
    #[default]
    Containers,
    Images,
    Settings,
    Logs,
}

pub struct Pages {
    route: Route,
    pub name: &'static str,
}

impl Pages {
    pub fn new(route: Route, name: &'static str) -> Self {
        Self { route, name }
    }
}
