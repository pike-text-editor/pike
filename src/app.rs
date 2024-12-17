use std::{io, path::PathBuf};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{self, Constraint, Direction, Layout},
    prelude::Backend,
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use crate::pike::Pike;

/// TUI application which displays the UI and handles events
#[allow(dead_code)]
pub struct App {
    exit: bool,
    backend: Pike,
}

#[allow(dead_code, unused_variables, unused_mut)]
impl App {
    pub fn build(args: Args) -> App {
        let cwd = std::env::current_dir().map_err(|_| "Failed to get current working directory");
        if cwd.is_err() {
            eprintln!("{}", cwd.err().unwrap());
            std::process::exit(1);
        }

        let config_path = args.config.map(PathBuf::from);
        let file_path = args.file.map(PathBuf::from);

        let backend: Result<Pike, String> =
            Pike::build(cwd.expect("Error case was handled"), file_path, config_path);

        match backend {
            Ok(backend) => App::new(backend),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }

    fn new(backend: Pike) -> App {
        App {
            exit: false,
            backend,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            if self.exit {
                return Ok(());
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(2)]);
        let main_area = layout.split(frame.area())[0];
        let status_bar_area = layout.split(frame.area())[1];
        self.render_buffer_contents(main_area, frame);
        self.render_status_bar(status_bar_area, frame);
    }

    /// Render the contents of the currently opened buffer in a given Rect
    fn render_buffer_contents(&self, area: layout::Rect, frame: &mut ratatui::Frame) {
        let contents = self.backend.current_buffer_contents();
        let text_widget = Text::from(contents);
        let paragraph_widget = Paragraph::new(text_widget).wrap(Wrap { trim: false });
        frame.render_widget(paragraph_widget, area);
    }

    /// Render the status bar in a given Rect
    fn render_status_bar(&self, area: layout::Rect, frame: &mut ratatui::Frame) {
        // TODO: come back to this when text insertion is implemented to display saved/unsaved
        // changes info
        let filename = self.backend.current_buffer_filename();
        let text_widget = Text::from(filename);

        let paragraph_widget = Paragraph::new(text_widget).wrap(Wrap { trim: false });
        let block_widget = paragraph_widget.block(Block::default().borders(Borders::TOP));

        frame.render_widget(block_widget, area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Paste(paste) => self.handle_paste_event(paste),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> Result<(), io::Error> {
        match event.kind {
            event::KeyEventKind::Press => self.handle_key_press(event),
            event::KeyEventKind::Release => Ok(()),
            event::KeyEventKind::Repeat => Ok(()),
        }
    }

    fn handle_paste_event(&self, contents: String) -> Result<(), io::Error> {
        todo!()
    }

    fn handle_mouse_event(&self, event: MouseEvent) -> Result<(), io::Error> {
        todo!()
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> Result<(), io::Error> {
        match key.code {
            KeyCode::Char('q') => {
                self.exit();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_key_release(&self, key: KeyEvent) -> Result<(), io::Error> {
        todo!()
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    /// Open a file finder by name in the current working directory
    fn find_files_in_cwd(&mut self) {
        todo!()
    }

    /// Open a text finder in the current working directory
    fn find_words_in_cwd() {
        todo!()
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Args {
    /// The configuration file to use
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[arg(value_name = "FILE")]
    file: Option<String>,
}

#[cfg(test)]
mod tests {

    use crate::test_util::temp_file_with_contents;

    use super::App;

    /// Create an App instance with a given file open
    fn app_with_file(filename: String) -> super::App {
        App::build(super::Args {
            config: None,
            file: Some(filename),
        })
    }

    /// Create an App instance with a file containing the given contents open
    fn app_with_file_contents(contents: String) -> super::App {
        let file = temp_file_with_contents(&contents);
        let filename = file.path().to_str().unwrap().to_string();
        app_with_file(filename)
    }

    #[test]
    fn test_display_buffer_contents() {
        let test_cases = [
            "Hello, world!",
            "Hello, world!\nThis is a test",
            r#"Hello, world!
            This is another test"#,
            "",
        ];

        for case in test_cases.iter() {
            let app = app_with_file_contents(case.to_string());
            let buffer_contents = app.backend.current_buffer_contents();
            assert_eq!(buffer_contents, *case);
        }
    }
}
