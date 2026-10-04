use color_eyre::{Report, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{self, EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

    let log_dir = match custom_log_dir {
        Some(path) => PathBuf::from(path),
        None => ProjectDirs::from("com", app_prefix, app_prefix)
            .map(|proj_dirs| proj_dirs.data_local_dir().join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs")),
    };

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::hourly(&log_dir, format!("{app_prefix}.log"));
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
