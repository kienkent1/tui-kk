mod app;
mod modules;
mod shared;
mod tests;
mod tuikk_core;
use crate::tuikk_core::{config::AppConfig, docker_conn::{DockerConfig, DockerConnection}};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    
    let app_config = AppConfig::load_or_default();

    AppConfig::init_global(app_config.clone());

    let _log_guard = tuikk_core::logging::init_logging(&app_config.log_level)?;
    DockerConnection::init_global(app_config.docker)?;

    match DockerConnection::ping().await {
        Ok(_) => tracing::info!("Connected to Docker successfully"),
        Err(e) => tracing::warn!("Docker is not running: {e}"),
    }


    let mut terminal = ratatui::init();
    let app_result = app::app(&mut terminal);

    ratatui::restore();

    app_result?;

    Ok(())
}
