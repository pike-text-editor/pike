mod app;
mod config;
mod key_shortcut;
mod operations;
mod pike;
mod test_util;
mod ui;
mod welcome_pike;

use clap::Parser;
use std::io;

use app::{App, Args};

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut terminal = ratatui::init();
    let mut app = App::build(args);
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
