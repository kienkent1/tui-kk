use ratatui::style::Color;
use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Debug, Clone, SmartDefault, Deserialize)]
#[serde(default)]
pub struct Themes {
    pub bg: Color,
    pub border: Color,
    pub text: Color,
    pub input: Color,
    pub input_focus: Color,
}
