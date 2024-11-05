mod app;
mod config;
mod operations;
mod pike;

use app::App;

fn main() {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.run(&mut terminal);

    ratatui::restore();
}
