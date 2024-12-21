use std::{io, path::PathBuf};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{self, Constraint, Direction, Layout, Position as TerminalPosition},
    prelude::Backend,
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Terminal,
};

use crate::{
    pike::Pike,
    ui::{BufferDisplay, UIState},
};

/// TUI application which displays the UI and handles events
#[allow(dead_code)]
pub struct App {
    exit: bool,
    backend: Pike,
    ui_state: UIState,
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
            ui_state: UIState::default(),
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

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(2)]);
        let area = frame.area();
        let main_area = layout.split(area)[0];
        let status_bar_area = layout.split(area)[1];
        self.render_buffer_contents(main_area, frame.buffer_mut());
        self.render_status_bar(status_bar_area, frame.buffer_mut());
        self.render_cursor(main_area, frame);
    }

    /// Render the contents of the currently opened buffer in a given Rect
    fn render_buffer_contents(&mut self, area: layout::Rect, buf: &mut ratatui::prelude::Buffer) {
        let buffer_contents = &self.backend.current_buffer_contents();
        let cursor_position = self.backend.cursor_position();
        let offset = &mut self.ui_state.buffer_offset;
        let buffer_widget = BufferDisplay::new(buffer_contents, cursor_position.as_ref(), offset);
        buffer_widget.render(area, buf);
    }

    /// Render the status bar in a given Rect
    fn render_status_bar(&self, area: layout::Rect, buf: &mut ratatui::prelude::Buffer) {
        // TODO: come back to this when text insertion is implemented to display saved/unsaved
        // changes info
        let filename = self.backend.current_buffer_filename();
        let text_widget = Text::from(filename);

        let paragraph_widget = Paragraph::new(text_widget).wrap(Wrap { trim: false });
        let block_widget = paragraph_widget.block(Block::default().borders(Borders::TOP));

        block_widget.render(area, buf);
    }

    /// Renders the cursor in the current buffer
    fn render_cursor(&mut self, area: layout::Rect, frame: &mut ratatui::prelude::Frame) {
        // TODO: probably should be split up so self is not mutable
        if let Some(position) = self.backend.cursor_position() {
            let cursor_position = self.calculate_cursor_render_position(area);
            frame.set_cursor_position(cursor_position);
        }
    }

    /// Get the position to render the cursor at in the current buffer.
    /// Subject to changing when handling more input scenarios, only works
    /// when editing the current buffer. Self has to be mutable here, since
    /// UIState is modified when calculating the cursor position
    pub fn calculate_cursor_render_position(&mut self, area: layout::Rect) -> TerminalPosition {
        // TODO: tidy this up
        let buffer_contents = &self.backend.current_buffer_contents();
        let cursor_position = self.backend.cursor_position();
        let offset = &mut self.ui_state.buffer_offset;

        let buffer_widget = BufferDisplay::new(buffer_contents, cursor_position.as_ref(), offset);
        buffer_widget.calculate_cursor_render_position(area)
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
            KeyCode::Left => {
                self.backend.move_cursor_left();
                Ok(())
            }
            KeyCode::Right => {
                self.backend.move_cursor_right();
                Ok(())
            }
            KeyCode::Up => {
                self.backend.move_cursor_up();
                Ok(())
            }
            KeyCode::Down => {
                self.backend.move_cursor_down();
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

    use ratatui::{
        buffer::Buffer,
        layout::{Position as TerminalPosition, Rect},
    };
    use tempfile::NamedTempFile;

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
    fn app_with_file_contents(contents: &str) -> super::App {
        let file = temp_file_with_contents(contents);
        let filename = file.path().to_str().unwrap().to_string();
        app_with_file(filename)
    }

    /// Return a string representation of a solid border of a given length.
    fn solid_border(length: usize) -> String {
        "─".repeat(length)
    }

    /// Return a string representation of a line filled with
    /// spaces of a given length
    fn n_spaces(n: usize) -> String {
        String::from(" ").repeat(n)
    }

    #[test]
    fn test_render_buffer_contents_fit() {
        let contents = String::from("Hello, world!");
        let mut app = app_with_file_contents(&contents);
        let width = 15;

        let mut buf = Buffer::empty(Rect::new(0, 0, width, 2));
        let expected = Buffer::with_lines(vec![contents, n_spaces(width.into())]);
        app.render_buffer_contents(buf.area, &mut buf);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_buffer_contents_too_long() {
        let contents = "Hello, world!";
        let mut app = app_with_file_contents(contents);
        let width = 4;
        let mut buf = Buffer::empty(Rect::new(0, 0, width, 1));
        let expected = Buffer::with_lines(vec!["Hell".to_string()]);
        app.render_buffer_contents(buf.area, &mut buf);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_status_bar() {
        // TODO: maybe factor this out to another function if needed later
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let file_path = file.path().to_str().unwrap().to_string();
        let filename = file.path().file_name().unwrap().to_str().unwrap();
        let app = app_with_file(file_path.clone());
        let width = 20;

        let mut buf = Buffer::empty(Rect::new(0, 0, width, 2));
        let expected = Buffer::with_lines(vec![solid_border(width.into()), filename.to_string()]);
        app.render_status_bar(buf.area, &mut buf);
        assert_eq!(buf, expected)
    }

    /// Helper function to assert the position to render the cursor at in the visible
    /// buffer
    fn assert_cursor_render_pos(app: &mut App, buf: &Buffer, expected: (u16, u16)) {
        let pos = app.calculate_cursor_render_position(buf.area);
        assert_eq!(pos, expected.into());
    }

    /// The cursor should not move past the bounds of the buffer
    #[test]
    fn test_cant_move_cursor_too_far_right() {
        let mut app = app_with_file_contents("t");
        let buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        // Starts at (0, 0)
        assert_cursor_render_pos(&mut app, &buf, (0, 0));

        app.backend.move_cursor_right();
        assert_cursor_render_pos(&mut app, &buf, (1, 0));

        app.backend.move_cursor_right();
        assert_cursor_render_pos(&mut app, &buf, (1, 0));
    }

    #[test]
    fn test_cant_move_cursor_too_far_down() {
        let mut app = app_with_file_contents("123");
        let buf = Buffer::empty(Rect::new(0, 0, 10, 10));

        app.backend.move_cursor_down();
        assert_cursor_render_pos(&mut app, &buf, (0, 0));
    }

    /// Helper function to verify cursor position and buffer rendering.
    fn assert_cursor_and_buffer(
        app: &mut App,
        buf: &mut Buffer,
        expected_cursor_pos: TerminalPosition,
        expected_lines: Vec<&str>,
    ) {
        // Verify cursor position.
        assert_cursor_render_pos(app, buf, expected_cursor_pos.into());

        // Verify buffer contents.
        let expected_buffer = Buffer::with_lines(
            expected_lines
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        );
        app.render_buffer_contents(buf.area, buf);
        assert_eq!(*buf, expected_buffer);
    }

    /// The buffer contents should shift right so that lines that
    /// are too long to render can be inspected by moving further right.
    #[test]
    fn test_buffer_shifts_when_moving_outside_visible_chars() {
        let mut app = app_with_file_contents("123\n456");
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 2));

        // Verify initial buffer rendering after the first cursor move.
        app.backend.move_cursor_right();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["2", "5"]);

        // Verify buffer rendering after the second cursor move.
        app.backend.move_cursor_right();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["3", "6"]);
    }

    /// When the buffer gets shifted right, it should not shift back
    /// left until the first displayed char is reached, only the visible
    /// cursor should be moved to the left
    #[test]
    fn test_buffer_does_not_shift_left_until_necessary() {
        let mut app = app_with_file_contents("1234");
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 1));
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["12"]);

        // Move the cursor to the last char, shifting the buffer
        app.backend.move_cursor_right();
        app.backend.move_cursor_right();
        app.backend.move_cursor_right();

        // Verify initial buffer rendering after the first cursor move.
        assert_cursor_and_buffer(&mut app, &mut buf, (1, 0).into(), vec!["34"]);

        // Move left
        app.backend.move_cursor_left();

        // The cursor should now point at 3 and be at (0, 0)
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["34"]);

        // Move left, the buffer should shift left
        app.backend.move_cursor_left();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["23"]);
    }

    /// The buffer contents should shift down so that lines that
    /// are too long to render can be inspected by moving further down.
    #[test]
    fn test_buffer_shifts_when_moving_outside_visible_lines() {
        let mut app = app_with_file_contents("123\n456\n789");
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));

        // Verify initial buffer rendering after the first cursor move.
        app.backend.move_cursor_down();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["456"]);

        // Verify buffer rendering after the second cursor move.
        app.backend.move_cursor_down();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["789"]);
    }

    /// When the buffer gets shifted down, it should not shift back
    /// up until the first displayed line is reached, only the visible
    /// cursor should be moved up
    #[test]
    fn test_buffer_does_not_shift_up_until_necessary() {
        let mut app = app_with_file_contents("123\n456\n789");
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 2));
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["123", "456"]);

        // Move the cursor to the last line, shifting the buffer
        app.backend.move_cursor_down();
        app.backend.move_cursor_down();

        // Verify initial buffer rendering after the first cursor move.
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 1).into(), vec!["456", "789"]);

        // Move up
        app.backend.move_cursor_up();

        // The cursor should now point at 4 and be at (0, 0)
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["456", "789"]);

        // Move up, the buffer should shift up
        app.backend.move_cursor_up();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0).into(), vec!["123", "456"]);
    }
}
