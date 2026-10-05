use ratatui::style::Color;
use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Debug, Clone, Copy, SmartDefault, Deserialize)]
#[serde(default)]
pub struct ThemeColor {
    pub bg: Color,
    pub border: Color,
    pub text: Color,
    pub input: Color,
    pub input_focus: Color,
}

impl ThemeColor {
    pub fn dark() -> Self {
        Self {
            bg: Color::Reset,
            border: Color::DarkGray,
            text: Color::White,
            input: Color::DarkGray,
            input_focus: Color::White,
        }
    }
}
