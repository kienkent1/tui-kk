use color_eyre::{Report, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{self, EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Resolve the log directory from user configuration (`AppConfig.log_dir`),
/// falling back to the platform XDG data directory or `./logs`.
pub fn resolve_log_dir(custom_log_dir: &Option<String>) -> PathBuf {
    let app_prefix = env!("APP_PREFIX");
    match custom_log_dir {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path.trim()),
        _ => ProjectDirs::from("com", app_prefix, app_prefix)
            .map(|proj_dirs| proj_dirs.data_local_dir().join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs")),
    }
}

/// Window
/// C:\Users\alice\AppData\Local\tuikk\tuikk\data\logs
/// Linux
/// /home/alice/.local/share/tuikk/logs or ~/.var/app/<app-id>/data/logs or /home/<username>/.local/share/tuikk/logs
/// Mac
/// /Users/alice/Library/Application Support/com.tuikk.tuikk/logs
pub fn init_logging(
    log_level: &str,
    custom_log_dir: &Option<String>,
) -> Result<WorkerGuard, Report> {
    let app_prefix = env!("APP_PREFIX");
    let log_dir = resolve_log_dir(custom_log_dir);

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        // No prefix, so the date comes first: "2026-10-10-14.<app_prefix>.log"
        .filename_suffix(format!("{app_prefix}.log"))
        .build(&log_dir)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true),
        )
        .init();

    Ok(guard)
}
