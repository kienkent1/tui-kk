/// Maximum number of time-series points kept in history for Dashboard charts.
pub const MAX_HISTORY_POINTS: usize = 60;

/// Number of TUI ticks between automatic Dashboard stats refreshes.
/// With default tick_rate = 4.0 Hz, 8 ticks = 2 seconds.
pub const DASHBOARD_REFRESH_TICKS: u64 = 8;

/// Byte unit thresholds for human-readable memory and storage formatting.
pub const KB: f64 = 1024.0;
pub const MB: f64 = KB * 1024.0;
pub const GB: f64 = MB * 1024.0;
pub const TB: f64 = GB * 1024.0;

