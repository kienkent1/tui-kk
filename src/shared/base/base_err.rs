use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BaseErr {
    // ==========================================
    // 1. Terminal / Backend Errors (Crossterm / Ratatui)
    // ==========================================
    #[error("Failed to initialize terminal: {0}")]
    TerminalInitFailed(#[source] std::io::Error),

    #[error("Failed to restore terminal state: {0}")]
    TerminalRestoreFailed(#[source] std::io::Error),

    #[error(
        "Terminal window size too small: current {width}x{height}, required min {min_width}x{min_height}"
    )]
    TerminalTooSmall {
        width: u16,
        height: u16,
        min_width: u16,
        min_height: u16,
    },

    // ==========================================
    // 2. I/O & File Operations (Logs, XDG Config, Dirs)
    // ==========================================
    #[error("Standard I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to resolve application directory: {0}")]
    DirectoryResolutionFailed(String),

    // ==========================================
    // 3. Configuration & Serialization
    // ==========================================
    #[error("Configuration file not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Failed to parse configuration: {reason}")]
    ConfigParseError { path: PathBuf, reason: String },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{resource_type} '{id}' not found")]
    NotFound { resource_type: &'static str, id: String },

    #[error("{resource_type} '{id}' already exists")]
    AlreadyExists { resource_type: &'static str, id: String },

    #[error("{resource_type} '{id}' is no longer available")]
    Gone { resource_type: &'static str, id: String },

    // ==========================================
    // Validation Errors  (~ HTTP 400 / 422)
    // ==========================================
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Validation failed for '{field}': {reason}")]
    UnprocessableEntity { field: &'static str, reason: String },

    #[error("Validation failed: {count} error(s)", count = errors.len())]
    ValidationErrors { errors: Vec<FieldError> },

    #[error("Operation '{operation}' not allowed: {reason}")]
    InvalidState { operation: &'static str, reason: String },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: &'static str },
}

#[derive(Debug)]
pub struct FieldError {
    pub field: &'static str,
    pub reason: String,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}