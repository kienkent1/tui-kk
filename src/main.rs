mod modules;
mod shared;
mod tests;
mod tuikk_core;
use crate::tuikk_core::{cli::Cli, config::AppConfig, docker_conn::DockerConnection};
use clap::Parser;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();
    let mut app_config = AppConfig::load_or_default();

    //===========Load cli================
    app_config.ui.frame_rate = args.frame_rate.unwrap_or(app_config.ui.frame_rate);
    app_config.ui.tick_rate = args.tick_rate.unwrap_or(app_config.ui.tick_rate);

    AppConfig::init_global(app_config.clone());

    let _log_guard = tuikk_core::logging::init_logging(&app_config.log_level)?;
    DockerConnection::init_global(app_config.docker)?;

    match DockerConnection::ping().await {
        Ok(_) => tracing::info!("Connected to Docker successfully"),
        Err(e) => tracing::warn!("Docker is not running: {e}"),
    }

    let mut terminal = ratatui::init();
    let app_result = tuikk_core::app::app(&mut terminal).await;

    ratatui::restore();

    app_result?;

    Ok(())
}
