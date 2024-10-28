mod app;
use std::io::Stdout;

use app::App;
use ratatui::prelude::CrosstermBackend;

fn main() {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.run(&mut terminal);

    ratatui::restore();
}
