use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Serialize, Deserialize, SmartDefault)]
#[serde(default)]
pub struct BaseFilter {
    #[default(20)]
    pub limit: i32,
    pub befor_id: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
    #[default(true)]
    pub is_desc: bool,
    pub filters: Option<HashMap<String, String>>,
}

impl BaseFilter {

}