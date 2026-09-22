mod app;
mod tuikk_core;
fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = app::app(&mut terminal);
    ratatui::restore();
    app_result
}
