mod app;
mod config;
mod operations;
mod pike;
mod ui;

use std::{env, io};

use app::App;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut terminal = ratatui::init();
    let mut app = App::build(&args);
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
