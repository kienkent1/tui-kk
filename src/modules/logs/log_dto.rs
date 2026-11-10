use std::path::PathBuf;

use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};

use crate::modules::logs::actions::LogButtonAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    #[default]
    Unknown,
}

impl LogLevel {
    pub fn parse(token: &str) -> Self {
        match token.trim().to_ascii_uppercase().as_str() {
            "ERROR" | "ERR" => Self::Error,
            "WARN" | "WARNING" => Self::Warn,
            "INFO" => Self::Info,
            "DEBUG" => Self::Debug,
            "TRACE" => Self::Trace,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
            Self::Unknown => "LOG",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn is_warn_or_error(self) -> bool {
        matches!(self, Self::Error | Self::Warn)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub line_number: usize,
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogFileSummary {
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified_epoch: u64,
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub last_error_preview: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRowTarget {
    pub row: usize,
    pub action: LogButtonAction,
}

#[derive(Clone, Debug)]
pub struct LogRowButton {
    pub target: LogRowTarget,
    pub rect: Rect,
}

/// Parse a `tracing_subscriber` log line into a structured `LogEntry`.
/// Expected format:
/// `2026-10-19T15:10:47.444708Z  WARN ThreadId(01) tui_kk::target: message`
pub fn parse_log_line(line_number: usize, line: &str) -> LogEntry {
    let raw = line.trim_end_matches(&['\r', '\n'][..]).to_owned();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return LogEntry {
            line_number,
            timestamp: String::new(),
            level: LogLevel::Unknown,
            target: String::new(),
            message: String::new(),
            raw,
        };
    }

    let mut parts = trimmed.split_whitespace();
    let Some(first) = parts.next() else {
        return LogEntry {
            line_number,
            timestamp: String::new(),
            level: LogLevel::Unknown,
            target: String::new(),
            message: trimmed.to_owned(),
            raw,
        };
    };

    let Some(second) = parts.next() else {
        return LogEntry {
            line_number,
            timestamp: String::new(),
            level: LogLevel::Unknown,
            target: String::new(),
            message: trimmed.to_owned(),
            raw,
        };
    };

    let level = LogLevel::parse(second);
    if level == LogLevel::Unknown {
        return LogEntry {
            line_number,
            timestamp: String::new(),
            level: LogLevel::Unknown,
            target: String::new(),
            message: trimmed.to_owned(),
            raw,
        };
    }

    // Remainder after timestamp and level
    let rest_after_level = trimmed
        .find(second)
        .map(|idx| trimmed[idx + second.len()..].trim_start())
        .unwrap_or("");

    // Skip optional `ThreadId(...)` token
    let after_thread = if rest_after_level.starts_with("ThreadId(") {
        rest_after_level
            .split_once(char::is_whitespace)
            .map(|(_, r)| r.trim_start())
            .unwrap_or(rest_after_level)
    } else {
        rest_after_level
    };

    let (target, message) = match after_thread.split_once(": ") {
        Some((t, m)) => (t.trim().to_owned(), m.to_owned()),
        None => (String::new(), after_thread.to_owned()),
    };

    LogEntry {
        line_number,
        timestamp: first.to_owned(),
        level,
        target,
        message,
        raw,
    }
}

