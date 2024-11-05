mod app;
mod config;
mod operations;
mod pike;

use std::io;

use app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
