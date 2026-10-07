use crate::tuikk_core::serdes::color_serde;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use serde_with::apply;
use smart_default::SmartDefault;
#[apply(Color => #[serde(with = "color_serde")])]
#[derive(Debug, Clone, Copy, SmartDefault, Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeColor {
    // ── Layout ───────────────────────────────────────────────
    /// Main application background
    #[default(Color::Reset)]
    pub bg: Color,
    /// Panel / sidebar background
    #[default(Color::Reset)]
    pub bg_panel: Color,
    /// Background of the selected row or item
    #[default(Color::Reset)]
    pub bg_selected: Color,

    // ── Border ───────────────────────────────────────────────
    /// Default border color
    #[default(Color::DarkGray)]
    pub border: Color,
    /// Border color when the panel is focused
    #[default(Color::White)]
    pub border_focus: Color,

    // ── Text ─────────────────────────────────────────────────
    /// Primary text color
    #[default(Color::White)]
    pub text: Color,
    /// Muted text — secondary labels, hints
    #[default(Color::DarkGray)]
    pub text_muted: Color,
    /// Text rendered on top of a selected background
    #[default(Color::Black)]
    pub text_selected: Color,

    // ── Input / Search bar ───────────────────────────────────
    /// Input field background (unfocused)
    #[default(Color::DarkGray)]
    pub input_bg: Color,
    /// Input field background when focused
    #[default(Color::White)]
    pub input_bg_focus: Color,
    /// Text color inside the input field
    #[default(Color::White)]
    pub input_text: Color,
    /// Placeholder text color
    #[default(Color::DarkGray)]
    pub input_placeholder: Color,
    /// Cursor color inside the input field
    #[default(Color::White)]
    pub input_cursor: Color,

    // ── Accent ───────────────────────────────────────────────
    /// Primary accent color — active tab, badge, highlight
    #[default(Color::White)]
    pub accent: Color,
    /// Text rendered on top of an accent background
    #[default(Color::White)]
    pub accent_text: Color,

    // ── Container status ─────────────────────────────────────
    /// Container is running
    #[default(Color::Green)]
    pub status_running: Color,
    /// Container is stopped / exited
    #[default(Color::Red)]
    pub status_stopped: Color,
    /// Container is paused
    #[default(Color::Yellow)]
    pub status_paused: Color,
    /// Container is dead
    #[default(Color::DarkGray)]
    pub status_dead: Color,
    /// Container is restarting
    #[default(Color::Cyan)]
    pub status_restarting: Color,

    // ── Severity / feedback ──────────────────────────────────
    /// Success state — operation completed
    #[default(Color::Green)]
    pub success: Color,
    /// Warning state — non-critical issue
    #[default(Color::Yellow)]
    pub warning: Color,
    /// Error state — operation failed
    #[default(Color::Red)]
    pub error: Color,
    /// Informational message
    #[default(Color::Cyan)]
    pub info: Color,

    // ── Scrollbar ────────────────────────────────────────────
    /// Scrollbar track (background rail)
    #[default(Color::DarkGray)]
    pub scrollbar_track: Color,
    /// Scrollbar thumb (draggable indicator)
    #[default(Color::White)]
    pub scrollbar_thumb: Color,
}

impl ThemeColor {
    pub fn default(theme: Option<String>) -> Self {
        match theme.as_deref() {
            Some("light") => ThemeColor::light(),
            _ => ThemeColor::dark(),
        }
    }

    pub fn dark() -> Self {
        Self {
            // Layout
            bg: Color::Rgb(18, 18, 24),
            bg_panel: Color::Rgb(26, 27, 38),
            bg_selected: Color::Rgb(40, 84, 140),

            // Border
            border: Color::Rgb(55, 58, 80),
            border_focus: Color::Rgb(82, 165, 245),

            // Text
            text: Color::Rgb(210, 215, 230),
            text_muted: Color::Rgb(100, 108, 135),
            text_selected: Color::Rgb(230, 235, 255),

            // Input
            input_bg: Color::Rgb(30, 32, 46),
            input_bg_focus: Color::Rgb(36, 40, 59),
            input_text: Color::Rgb(210, 215, 230),
            input_placeholder: Color::Rgb(80, 88, 110),
            input_cursor: Color::Rgb(130, 200, 255),

            // Accent — cyan / blue
            accent: Color::Rgb(82, 165, 245),
            accent_text: Color::Rgb(18, 18, 24),

            // Container status
            status_running: Color::Rgb(82, 215, 135),
            status_stopped: Color::Rgb(240, 90, 90),
            status_paused: Color::Rgb(240, 185, 70),
            status_dead: Color::Rgb(90, 90, 105),
            status_restarting: Color::Rgb(80, 190, 240),

            // Severity
            success: Color::Rgb(82, 215, 135),
            warning: Color::Rgb(240, 185, 70),
            error: Color::Rgb(240, 90, 90),
            info: Color::Rgb(80, 190, 240),

            // Scrollbar
            scrollbar_track: Color::Rgb(30, 32, 46),
            scrollbar_thumb: Color::Rgb(70, 76, 110),
        }
    }

    pub fn light() -> Self {
        Self {
            // Layout — white / light blue
            bg: Color::Rgb(240, 244, 252),
            bg_panel: Color::Rgb(228, 235, 250),
            bg_selected: Color::Rgb(186, 213, 248),

            // Border
            border: Color::Rgb(185, 200, 225),
            border_focus: Color::Rgb(30, 110, 210),

            // Text
            text: Color::Rgb(28, 35, 58),
            text_muted: Color::Rgb(115, 130, 165),
            text_selected: Color::Rgb(15, 30, 70),

            // Input
            input_bg: Color::Rgb(255, 255, 255),
            input_bg_focus: Color::Rgb(245, 250, 255),
            input_text: Color::Rgb(28, 35, 58),
            input_placeholder: Color::Rgb(150, 165, 195),
            input_cursor: Color::Rgb(30, 110, 210),

            // Accent — deep blue
            accent: Color::Rgb(30, 110, 210),
            accent_text: Color::Rgb(255, 255, 255),

            // Container status
            status_running: Color::Rgb(25, 155, 85),
            status_stopped: Color::Rgb(200, 50, 50),
            status_paused: Color::Rgb(185, 135, 20),
            status_dead: Color::Rgb(155, 160, 175),
            status_restarting: Color::Rgb(20, 140, 200),

            // Severity
            success: Color::Rgb(25, 155, 85),
            warning: Color::Rgb(185, 135, 20),
            error: Color::Rgb(200, 50, 50),
            info: Color::Rgb(20, 140, 200),

            // Scrollbar
            scrollbar_track: Color::Rgb(215, 222, 240),
            scrollbar_thumb: Color::Rgb(150, 170, 210),
        }
    }
}
