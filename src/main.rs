mod modules;
mod shared;
mod tests;
mod tuikk_core;
use crate::tuikk_core::{app::App, cli::Cli, config::AppConfig, docker_conn::DockerConnection, tui::Tui};
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



    let _log_guard = tuikk_core::logging::init_logging(&app_config.log_level)?;

    let tick_rate = app_config.ui.tick_rate;
    let frame_rate = app_config.ui.frame_rate;


    AppConfig::init_global(app_config.clone());
    DockerConnection::init_global(app_config.docker)?;

    match DockerConnection::ping().await {
        Ok(_) => tracing::info!("Connected to Docker successfully"),
        Err(e) => tracing::warn!("Docker is not running: {e}"),
    }

    let mut tui = Tui::new(tick_rate, frame_rate).mouse(true).paste(true);
    let mut app = App::new();
    app.run(&mut tui).await?;


    Ok(())
}
