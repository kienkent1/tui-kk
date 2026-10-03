use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Serialize, Deserialize, SmartDefault)]
#[serde(default)]
pub struct BaseFilter {
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub filters: Option<HashMap<String, Vec<String>>>,
}