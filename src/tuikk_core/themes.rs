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

    // ── Tab bar ──────────────────────────────────────────────
    /// Background of the active tab
    #[default(Color::Reset)]
    pub tab_active_bg: Color,
    /// Text color of the active tab
    #[default(Color::White)]
    pub tab_active_text: Color,
    /// Background of an inactive tab
    #[default(Color::Reset)]
    pub tab_inactive_bg: Color,
    /// Text color of an inactive tab
    #[default(Color::DarkGray)]
    pub tab_inactive_text: Color,

    // ── Sidebar ──────────────────────────────────────────────
    /// Sidebar background (may differ from bg_panel)
    #[default(Color::Reset)]
    pub sidebar_bg: Color,
    /// Sidebar item text (normal)
    #[default(Color::White)]
    pub sidebar_text: Color,
    /// Sidebar item text when hovered / highlighted
    #[default(Color::White)]
    pub sidebar_text_active: Color,
    /// Background of the active sidebar item
    #[default(Color::Reset)]
    pub sidebar_active_bg: Color,
    /// Sidebar section title / category label
    #[default(Color::DarkGray)]
    pub sidebar_section: Color,

    // ── List / Table ─────────────────────────────────────────
    /// Alternating row background (even rows)
    #[default(Color::Reset)]
    pub list_row_even: Color,
    /// Alternating row background (odd rows)
    #[default(Color::Reset)]
    pub list_row_odd: Color,
    /// Column header text
    #[default(Color::White)]
    pub list_header_text: Color,
    /// Column header background
    #[default(Color::Reset)]
    pub list_header_bg: Color,

    // ── Modal / Dialog ───────────────────────────────────────
    /// Modal dialog background
    #[default(Color::Reset)]
    pub modal_bg: Color,
    /// Modal border color
    #[default(Color::White)]
    pub modal_border: Color,
    /// Semi-transparent overlay behind modal (fill char color)
    #[default(Color::DarkGray)]
    pub modal_overlay: Color,

    // ── Tooltip / Popup ──────────────────────────────────────
    /// Tooltip / floating popup background
    #[default(Color::Reset)]
    pub tooltip_bg: Color,
    /// Tooltip text color
    #[default(Color::White)]
    pub tooltip_text: Color,

    // ── Keybinding hint bar ──────────────────────────────────
    /// Background of the bottom keybinding hint bar
    #[default(Color::Reset)]
    pub keybind_bar_bg: Color,
    /// Key label color (e.g. "q")
    #[default(Color::White)]
    pub keybind_key: Color,
    /// Description text next to the key label (e.g. "Quit")
    #[default(Color::DarkGray)]
    pub keybind_desc: Color,
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
            // Layout — ayu dark
            bg: Color::Rgb(16, 20, 28),           // #10141c — editor bg
            bg_panel: Color::Rgb(20, 24, 33),     // #141821 — panel / widget bg
            bg_selected: Color::Rgb(71, 82, 102), // #475266 — list selection (opaque)

            // Border
            border: Color::Rgb(27, 31, 41),         // #1b1f29
            border_focus: Color::Rgb(230, 180, 80), // #e6b450 — ayu gold accent

            // Text
            text: Color::Rgb(191, 189, 182), // #bfbdb6 — primary foreground
            text_muted: Color::Rgb(90, 99, 120), // #5a6378 — muted / secondary
            text_selected: Color::Rgb(191, 189, 182), // same as text on selection bg

            // Input
            input_bg: Color::Rgb(16, 20, 28),           // #10141c
            input_bg_focus: Color::Rgb(20, 24, 33),     // #141821 slightly lighter
            input_text: Color::Rgb(191, 189, 182),      // #bfbdb6
            input_placeholder: Color::Rgb(90, 99, 120), // #5a637880 muted
            input_cursor: Color::Rgb(230, 180, 80),     // #e6b450 — gold cursor

            // Accent — ayu signature gold
            accent: Color::Rgb(230, 180, 80),     // #e6b450
            accent_text: Color::Rgb(118, 91, 36), // #765b24 — dark text on gold bg

            // Container status
            status_running: Color::Rgb(112, 191, 86), // #70bf56 green
            status_stopped: Color::Rgb(242, 109, 120), // #f26d78 red
            status_paused: Color::Rgb(255, 180, 84),  // #ffb454 yellow
            status_dead: Color::Rgb(90, 99, 120),     // #5a6378 muted
            status_restarting: Color::Rgb(57, 186, 230), // #39bae6 cyan

            // Severity
            success: Color::Rgb(112, 191, 86), // #70bf56
            warning: Color::Rgb(230, 180, 80), // #e6b450
            error: Color::Rgb(217, 87, 87),    // #d95757
            info: Color::Rgb(57, 186, 230),    // #39bae6

            // Scrollbar
            scrollbar_track: Color::Rgb(16, 20, 28), // #10141c — same as bg
            scrollbar_thumb: Color::Rgb(90, 99, 120), // #5a637866 opaque

            // Tab bar
            tab_active_bg: Color::Rgb(16, 20, 28), // #10141c
            tab_active_text: Color::Rgb(191, 189, 182), // #bfbdb6
            tab_inactive_bg: Color::Rgb(13, 16, 23), // #0d1017
            tab_inactive_text: Color::Rgb(90, 99, 120), // #5a6378

            // Sidebar
            sidebar_bg: Color::Rgb(13, 16, 23),    // #0d1017
            sidebar_text: Color::Rgb(90, 99, 120), // #5a6378
            sidebar_text_active: Color::Rgb(191, 189, 182), // #bfbdb6
            sidebar_active_bg: Color::Rgb(71, 82, 102), // #475266
            sidebar_section: Color::Rgb(90, 99, 120), // #5a6378

            // List / Table
            list_row_even: Color::Rgb(16, 20, 28), // #10141c
            list_row_odd: Color::Rgb(13, 16, 23),  // #0d1017
            list_header_text: Color::Rgb(230, 180, 80), // #e6b450 gold
            list_header_bg: Color::Rgb(20, 24, 33), // #141821

            // Modal / Dialog
            modal_bg: Color::Rgb(20, 24, 33),       // #141821
            modal_border: Color::Rgb(230, 180, 80), // #e6b450
            modal_overlay: Color::Rgb(0, 0, 0),     // near-black

            // Tooltip / Popup
            tooltip_bg: Color::Rgb(20, 24, 33),      // #141821
            tooltip_text: Color::Rgb(191, 189, 182), // #bfbdb6

            // Keybinding hint bar
            keybind_bar_bg: Color::Rgb(1, 1, 2), // #010102 — status bar bg
            keybind_key: Color::Rgb(230, 180, 80), // #e6b450 gold
            keybind_desc: Color::Rgb(90, 99, 120), // #5a6378
        }
    }

    pub fn light() -> Self {
        Self {
            // Layout — ayu light
            bg: Color::Rgb(252, 252, 252),       // #fcfcfc — editor bg
            bg_panel: Color::Rgb(248, 249, 250), // #f8f9fa — surface / panel
            bg_selected: Color::Rgb(107, 125, 143), // #6b7d8f24 opaque selection

            // Border
            border: Color::Rgb(224, 228, 231), // #e0e4e7 — surface.border
            border_focus: Color::Rgb(242, 151, 24), // #f29718 — ayu orange accent

            // Text
            text: Color::Rgb(92, 97, 102), // #5c6166 — primary fg
            text_muted: Color::Rgb(130, 142, 159), // #828e9f — secondary / muted
            text_selected: Color::Rgb(92, 97, 102), // same on selection bg

            // Input
            input_bg: Color::Rgb(252, 252, 252),       // #fcfcfc
            input_bg_focus: Color::Rgb(248, 249, 250), // #f8f9fa slightly darker
            input_text: Color::Rgb(92, 97, 102),       // #5c6166
            input_placeholder: Color::Rgb(130, 142, 159), // #828e9f
            input_cursor: Color::Rgb(242, 151, 24),    // #f29718 orange cursor

            // Accent — ayu orange
            accent: Color::Rgb(242, 151, 24),    // #f29718
            accent_text: Color::Rgb(126, 75, 1), // #7e4b01 — dark brown on orange

            // Container status
            status_running: Color::Rgb(108, 191, 67), // #6cbf43 green
            status_stopped: Color::Rgb(230, 80, 80),  // #e65050 red
            status_paused: Color::Rgb(235, 164, 0),   // #eba400 amber
            status_dead: Color::Rgb(130, 142, 159),   // #828e9f muted
            status_restarting: Color::Rgb(85, 180, 212), // #55b4d4 cyan

            // Severity
            success: Color::Rgb(108, 191, 67), // #6cbf43
            warning: Color::Rgb(235, 164, 0),  // #eba400
            error: Color::Rgb(230, 80, 80),    // #e65050
            info: Color::Rgb(85, 180, 212),    // #55b4d4

            // Scrollbar
            scrollbar_track: Color::Rgb(248, 249, 250), // #f8f9fa
            scrollbar_thumb: Color::Rgb(130, 142, 159), // #828e9f66 opaque

            // Tab bar
            tab_active_bg: Color::Rgb(252, 252, 252), // #fcfcfc — active tab bg
            tab_active_text: Color::Rgb(92, 97, 102), // #5c6166
            tab_inactive_bg: Color::Rgb(241, 242, 244), // #f1f2f4 — tab strip bg
            tab_inactive_text: Color::Rgb(130, 142, 159), // #828e9f

            // Sidebar
            sidebar_bg: Color::Rgb(248, 249, 250),   // #f8f9fa
            sidebar_text: Color::Rgb(130, 142, 159), // #828e9f
            sidebar_text_active: Color::Rgb(92, 97, 102), // #5c6166
            sidebar_active_bg: Color::Rgb(107, 125, 143), // #6b7d8f24 opaque
            sidebar_section: Color::Rgb(130, 142, 159), // #828e9f

            // List / Table
            list_row_even: Color::Rgb(252, 252, 252), // #fcfcfc
            list_row_odd: Color::Rgb(248, 249, 250),  // #f8f9fa
            list_header_text: Color::Rgb(242, 151, 24), // #f29718 orange
            list_header_bg: Color::Rgb(241, 242, 244), // #f1f2f4

            // Modal / Dialog
            modal_bg: Color::Rgb(250, 250, 250), // #fafafa — widget bg
            modal_border: Color::Rgb(242, 151, 24), // #f29718
            modal_overlay: Color::Rgb(107, 125, 143), // #6b7d8f muted overlay

            // Tooltip / Popup
            tooltip_bg: Color::Rgb(250, 250, 250), // #fafafa
            tooltip_text: Color::Rgb(92, 97, 102), // #5c6166

            // Keybinding hint bar
            keybind_bar_bg: Color::Rgb(235, 238, 240), // #ebeef0 — status bar bg
            keybind_key: Color::Rgb(242, 151, 24),     // #f29718 orange
            keybind_desc: Color::Rgb(130, 142, 159),   // #828e9f
        }
    }
}
