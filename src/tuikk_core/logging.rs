use color_eyre::{Report, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{self, EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(log_level: &str) -> Result<WorkerGuard, Report> {
    let app_prefix = env!("APP_PREFIX");
    let log_dir = if let Some(proj_dirs) = ProjectDirs::from("com", &app_prefix, &app_prefix) {
        proj_dirs.data_local_dir().join("logs")
    } else {
        PathBuf::from("logs")
    };

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, format!("{app_prefix}.log"));
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

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
