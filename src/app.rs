use std::{io, path::PathBuf};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    widgets::Block,
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
        let cwd = std::env::current_dir().expect("Failed to get current working directory");

        let config_path = args.config.map(PathBuf::from);
        let file_path = args.file.map(PathBuf::from);

        let backend: Result<Pike, String> = Pike::build(cwd, file_path, config_path);

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
            .constraints([Constraint::Min(1)]);
        let area = layout.split(frame.area())[0];
        let block = Block::default().title("Hello pike");
        frame.render_widget(block, area);
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
