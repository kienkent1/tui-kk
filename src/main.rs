mod app;
mod shared;
mod tuikk_core;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let _log_guard = tuikk_core::logging::init_logging()?;

    let mut terminal = ratatui::init();
    let app_result = app::app(&mut terminal);

    ratatui::restore();

    app_result?;

    Ok(())
}
