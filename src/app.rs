use std::{env, io, path::PathBuf, process, rc::Rc};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position as TerminalPosition, Rect},
    prelude::{Backend, StatefulWidget},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Terminal,
};

use crate::{
    operations::Operation,
    pike::Pike,
    ui::{
        BufferDisplayOffset, BufferDisplayState, BufferDisplayWidget, CursorCalculationMode,
        FileInput, FileInputRole, UIState,
    },
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
        let cwd = env::current_dir().map_err(|_| "Failed to get current working directory");
        if cwd.is_err() {
            eprintln!("{}", cwd.err().unwrap());
            process::exit(1);
        }

        let config_path = args.config.map(PathBuf::from);
        let file_path = args.file.map(PathBuf::from);

        let backend: Result<Pike, String> =
            Pike::build(cwd.expect("Error case was handled"), file_path, config_path);

        match backend {
            Ok(backend) => App::new(backend),
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    }

    fn new(backend: Pike) -> App {
        let offset = BufferDisplayOffset::default();
        let buffer_state = BufferDisplayState::new(offset);
        let file_input = None;
        let ui_state = UIState {
            buffer_state,
            file_input,
        };

        App {
            exit: false,
            backend,
            ui_state,
        }
    }

    /// Builds an app with the default configuration and no open file
    fn build_default() -> Self {
        App::build(Args {
            config: None,
            file: None,
        })
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
        let layout = self.split_area(frame.area());

        let main_area = layout[0];
        let status_bar_area = layout[1];

        let cursor_pos = self.backend.cursor_position();

        let mut render_cursor_position;

        self.render_buffer_contents(main_area, frame.buffer_mut());

        let file_input_value = self.ui_state.file_input.clone();

        if let Some(ref input_state) = file_input_value {
            self.render_file_input(status_bar_area, frame.buffer_mut());
            render_cursor_position = self.ui_state.calculate_cursor_position(
                CursorCalculationMode::FileInput(&input_state.input),
                &layout,
                cursor_pos,
            );
        } else {
            render_cursor_position = self.ui_state.calculate_cursor_position(
                CursorCalculationMode::Buffer,
                &layout,
                cursor_pos,
            );
            self.render_status_bar(status_bar_area, frame.buffer_mut());
        }

        self.render_cursor(frame, render_cursor_position);
    }

    /// Splits an area using the main app layout and returns the
    /// resulting areas
    pub fn split_area(&self, area: Rect) -> Rc<[Rect]> {
        let file_input_open = self.ui_state.file_input.is_some();

        // if a file input is rendered in the status bar, an additional border
        // is rendered
        let status_bar_height = if file_input_open { 3 } else { 2 };

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(status_bar_height)])
            .split(area)
    }

    /// Render the contents of the currently opened buffer in a given Rect
    fn render_buffer_contents(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let contents = self.backend.current_buffer_contents();
        let cursor = self.backend.cursor_position();

        let widget = BufferDisplayWidget::new(&contents, cursor);
        widget.render(area, buf, &mut self.ui_state.buffer_state);
    }

    /// Render the status bar in a given Rect
    fn render_status_bar(&self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let filename = self.backend.current_buffer_filename();
        let is_modified = self.backend.is_current_buffer_modified();

        let indicator = if is_modified { "*" } else { "" };
        let text_widget = Text::from(format!("{}{}", filename, indicator));

        let paragraph_widget = Paragraph::new(text_widget).wrap(Wrap { trim: false });
        let block_widget = paragraph_widget.block(Block::default().borders(Borders::TOP));

        block_widget.render(area, buf);
    }

    /// Render the cursor in a given position
    fn render_cursor(&self, frame: &mut ratatui::prelude::Frame, position: TerminalPosition) {
        frame.set_cursor_position(position);
    }

    /// Render the file input in a given Rect
    fn render_file_input(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        FileInput::default().render(
            area,
            buf,
            self.ui_state
                .file_input
                .as_mut()
                .expect("None case was handled"),
        );
    }

    /// Open a file input with the given contents and store it in UIState
    fn open_file_input(&mut self, contents: &str, role: FileInputRole) {
        self.ui_state.file_input = Some((contents, role).into());
    }

    /// Close the currently open file input
    fn close_file_input(&mut self) {
        self.ui_state.file_input = None;
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

    /// Try to handle the key press using a file input. Returns a boolean
    /// indicating whether the event has been handled or not.
    fn try_handle_key_press_with_file_input(&mut self, key: KeyEvent) -> bool {
        // No input means the event can't be handled
        let input = match self.ui_state.file_input.as_mut() {
            Some(input) => input,
            None => return false,
        };

        // Perform the corresponding operation and close the input
        if (key.code, key.modifiers) == (KeyCode::Enter, KeyModifiers::NONE) {
            let path = input.to_path();
            match input.role {
                FileInputRole::GetOpenPath => self.open_file_from_path(path),
                FileInputRole::GetSavePath => {
                    self.backend.bind_current_buffer_to_path(path);
                    self.handle_save_operation();
                }
            }

            self.close_file_input();
            return true;
        }

        // Close the input
        if (key.code, key.modifiers) == (KeyCode::Esc, KeyModifiers::NONE) {
            self.close_file_input();
            return true;
        }

        // Try to create a request to the file input and handle it
        match Self::key_event_to_input_request(key) {
            Some(request) => {
                input.handle(request);
                true
            }
            None => false,
        }
    }

    fn open_file_from_path(&mut self, path: PathBuf) {
        self.backend
            .create_and_open_file(&path)
            // TODO: display message in the UI
            .expect("Error opening file!");
    }

    /// Try to convert a given key event to an InputRequest to be sent to a tui_input::Input
    /// instance.
    fn key_event_to_input_request(key: KeyEvent) -> Option<tui_input::InputRequest> {
        match (key.code, key.modifiers) {
            (KeyCode::Char(chr), KeyModifiers::NONE) => {
                Some(tui_input::InputRequest::InsertChar(chr))
            }
            (KeyCode::Char(chr), KeyModifiers::SHIFT) => {
                Some(tui_input::InputRequest::InsertChar(chr))
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                Some(tui_input::InputRequest::DeletePrevChar)
            }
            (KeyCode::Delete, KeyModifiers::NONE) => Some(tui_input::InputRequest::DeleteNextChar),
            (KeyCode::Left, KeyModifiers::NONE) => Some(tui_input::InputRequest::GoToPrevChar),
            (KeyCode::Right, KeyModifiers::NONE) => Some(tui_input::InputRequest::GoToNextChar),
            _ => None,
        }
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> Result<(), io::Error> {
        if self.try_handle_key_press_with_file_input(key) {
            return Ok(());
        }

        if self.try_handle_keybind(key) {
            return Ok(());
        }

        if self.try_handle_input_key(key) && !key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(());
        }

        match key.code {
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.backend.move_cursor_left_by_word();
                } else {
                    self.backend.move_cursor_left();
                }
                Ok(())
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.backend.move_cursor_right_by_word();
                } else {
                    self.backend.move_cursor_right();
                }
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
            KeyCode::End => {
                self.backend.move_cursor_to_end_of_line();
                Ok(())
            }
            KeyCode::Home => {
                self.backend.move_cursor_to_start_of_line();
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

    /// Tries to match the given key event to a registered keybind and handle it.
    fn try_handle_keybind(&mut self, key: KeyEvent) -> bool {
        match self.backend.get_keymap(&key.into()).cloned() {
            Some(op) => {
                self.handle_operation(&op);
                true
            }
            None => false,
        }
    }

    fn try_handle_input_key(&mut self, key: KeyEvent) -> bool {
        // TODO: Better error handling
        if let KeyCode::Char(ch) = key.code {
            let _ = self.backend.write_to_current_buffer(&ch.to_string());

            return true;
        }
        match key.code {
            KeyCode::Enter => {
                let _ = self.backend.write_to_current_buffer("\n");
                true
            }
            KeyCode::Backspace => {
                self.backend.delete_character_from_current_buffer();
                true
            }
            _ => false,
        }
    }

    fn handle_operation(&mut self, op: &Operation) {
        match op {
            Operation::OpenFile => {
                self.open_file_input("", FileInputRole::GetOpenPath);
            }
            Operation::Quit => {
                self.exit();
            }
            Operation::CreateNewBuffer => self.backend.open_new_buffer(),
            Operation::SwitchToPreviousBuffer => self.backend.previous_buffer(),
            Operation::SwitchToNextBuffer => self.backend.next_buffer(),
            Operation::SaveBufferToFile => self.handle_save_operation(),

            Operation::SearchInCurrentBuffer => todo!("Handle SearchInCurrentBuffer operation"),
            Operation::SearchAndReplaceInCurrentBuffer => {
                todo!("Handle SearchAndReplaceInCurrentBuffer operation")
            }

            Operation::Undo => self.backend.undo(),
            Operation::Redo => self.backend.redo(),

            // WARN: these probably won't be supported
            Operation::FindFilesInCWD => todo!("Handle FindFilesInCWD operation"),
            Operation::FindTextInCWD => todo!("Handle FindTextInCWD operation"),
            Operation::OpenBufferPicker => todo!("Handle OpenBufferPicker operation"),
        }
    }

    fn handle_save_operation(&mut self) {
        if let Some(path) = self.backend.current_buffer_path() {
            if let Err(err) = self.backend.save_current_buffer() {
                eprintln!("Failed to save buffer: {}", err);
            }
        } else {
            // Ask for filepath if the buffer is not bound to one
            self.open_file_input("", FileInputRole::GetSavePath);
        }
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

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{buffer::Buffer, layout::Rect};
    use tempfile::NamedTempFile;
    use tui_input::InputRequest;

    use crate::{
        operations::Operation,
        test_util::{
            temp_file_with_contents,
            ui::{n_spaces, solid_border},
        },
        ui::FileInputRole,
    };

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

    /// Create an App instance with a given config
    fn app_with_config(config_contents: &str) -> App {
        let config_file = temp_file_with_contents(config_contents);
        let filename = config_file.path().to_str().unwrap().to_string();
        App::build(super::Args {
            config: Some(filename),
            file: None,
        })
    }

    /// Used in unit tests to provide the UI element, based on which the cursor
    /// position should be calculated, so that a testing buffer can be created only
    /// to accommodate this element instead of the whole UI.
    enum CursorRenderingWidget {
        CurrentBuffer,
        FileInput,
    }

    /// Helper function to assert the position to render the cursor at in the visible
    /// buffer
    fn assert_cursor_render_pos(
        app: &mut App,
        buf: &ratatui::buffer::Buffer,
        renderer: CursorRenderingWidget,
        expected: (u16, u16),
    ) {
        let pos = match renderer {
            CursorRenderingWidget::CurrentBuffer => {
                let cursor_position = app.backend.cursor_position();

                if let Some(cp) = cursor_position {
                    // Scroll horizontally
                    app.ui_state
                        .buffer_state
                        .update_x_offset(buf.area, cp.offset);
                    // Scroll vertically
                    app.ui_state.buffer_state.update_y_offset(buf.area, cp.line);
                }

                // 3) Ask UIState where the cursor _should_ be rendered:
                app.ui_state
                    .calculate_cursor_for_buffer(buf.area, cursor_position)
            }

            CursorRenderingWidget::FileInput => {
                let input = app
                    .ui_state
                    .file_input
                    .as_ref()
                    .expect("A file input should be open when testing cursor in file input");
                app.ui_state
                    .calculate_cursor_for_file_input(&input.input, buf.area)
            }
        };

        assert_eq!(pos, expected.into());
    }

    /// Shorthand for defining the renderer in unit tests and calling assert_cursor_render_pos
    fn acrp_based_on_current_buffer(
        app: &mut App,
        buf: &ratatui::buffer::Buffer,
        expected: (u16, u16),
    ) {
        assert_cursor_render_pos(app, buf, CursorRenderingWidget::CurrentBuffer, expected);
    }

    fn acrp_based_on_file_input(app: &mut App, buf: &Buffer, expected: (u16, u16)) {
        assert_cursor_render_pos(app, buf, CursorRenderingWidget::FileInput, expected);
    }

    /// Helper function to verify cursor position and buffer rendering.
    fn assert_cursor_and_buffer(
        app: &mut App,
        buf: &mut Buffer,
        expected_cursor_pos: (u16, u16),
        expected_lines: Vec<&str>,
    ) {
        // Verify cursor position.
        acrp_based_on_current_buffer(app, buf, expected_cursor_pos);

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

    #[allow(dead_code)]
    /// Helper function to assert the position to render the cursor at in the visible
    /// buffer after syncing the buffer contents and cursor position from the backend.
    fn assert_cursor_render_pos_no_input(app: &mut App, buf: &Buffer, expected: (u16, u16)) {
        let cursor_position = app.backend.cursor_position();

        if let Some(cp) = cursor_position {
            app.ui_state
                .buffer_state
                .update_x_offset(buf.area, cp.offset);
            app.ui_state.buffer_state.update_y_offset(buf.area, cp.line);
        }

        let pos = app
            .ui_state
            .calculate_cursor_for_buffer(buf.area, cursor_position);

        assert_eq!(pos, expected.into());
    }
    /// The cursor should not move past the bounds of the buffer
    #[test]
    fn test_cant_move_cursor_too_far_right() {
        let mut app = app_with_file_contents("t");
        let buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        // Starts at (0, 0)
        acrp_based_on_current_buffer(&mut app, &buf, (0, 0));

        app.backend.move_cursor_right();
        acrp_based_on_current_buffer(&mut app, &buf, (1, 0));

        app.backend.move_cursor_right();
        acrp_based_on_current_buffer(&mut app, &buf, (1, 0));
    }

    #[test]
    fn test_cant_move_cursor_too_far_down() {
        let mut app = app_with_file_contents("123");
        let buf = Buffer::empty(Rect::new(0, 0, 10, 10));

        app.backend.move_cursor_down();
        acrp_based_on_current_buffer(&mut app, &buf, (0, 0));

        app.backend.move_cursor_down();
        acrp_based_on_current_buffer(&mut app, &buf, (0, 0));
    }

    /// The buffer contents should shift right so that lines that
    /// are too long to render can be inspected by moving further right.
    #[test]
    fn test_buffer_shifts_when_moving_outside_visible_chars() {
        let mut app = app_with_file_contents("123\n456");
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 2));

        // Verify initial buffer rendering after the first cursor move.
        app.backend.move_cursor_right();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["2", "5"]);

        // Verify buffer rendering after the second cursor move.
        app.backend.move_cursor_right();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["3", "6"]);
    }

    /// When the buffer gets shifted right, it should not shift back
    /// left until the first displayed char is reached, only the visible
    /// cursor should be moved to the left
    #[test]
    fn test_buffer_does_not_shift_left_until_necessary() {
        let mut app = app_with_file_contents("1234");
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 1));
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["12"]);

        // Move the cursor to the last char, shifting the buffer
        app.backend.move_cursor_right();
        app.backend.move_cursor_right();
        app.backend.move_cursor_right();

        // Verify initial buffer rendering after the first cursor move.
        assert_cursor_and_buffer(&mut app, &mut buf, (1, 0), vec!["34"]);

        // Move left
        app.backend.move_cursor_left();

        // The cursor should now point at 3 and be at (0, 0)
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["34"]);

        // Move left, the buffer should shift left
        app.backend.move_cursor_left();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["23"]);
    }

    /// The buffer contents should shift down so that lines that
    /// are too long to render can be inspected by moving further down.
    #[test]
    fn test_buffer_shifts_when_moving_outside_visible_lines() {
        let mut app = app_with_file_contents("123\n456\n789");
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));

        // Verify initial buffer rendering after the first cursor move.
        app.backend.move_cursor_down();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["456"]);

        // Verify buffer rendering after the second cursor move.
        app.backend.move_cursor_down();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["789"]);
    }

    /// When the buffer gets shifted down, it should not shift back
    /// up until the first displayed line is reached, only the visible
    /// cursor should be moved up
    #[test]
    fn test_buffer_does_not_shift_up_until_necessary() {
        let mut app = app_with_file_contents("123\n456\n789");
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 2));
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["123", "456"]);

        // Move the cursor to the last line, shifting the buffer
        app.backend.move_cursor_down();
        app.backend.move_cursor_down();

        // Verify initial buffer rendering after the first cursor move.
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 1), vec!["456", "789"]);

        // Move up
        app.backend.move_cursor_up();

        // The cursor should now point at 4 and be at (0, 0)
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["456", "789"]);

        // Move up, the buffer should shift up
        app.backend.move_cursor_up();
        assert_cursor_and_buffer(&mut app, &mut buf, (0, 0), vec!["123", "456"]);
    }

    #[test]
    fn test_cursor_position_file_input() {
        let mut app = app_with_file_contents("");
        let buf = Buffer::empty(Rect::new(0, 0, 10, 3));

        app.open_file_input("", FileInputRole::GetOpenPath);
        acrp_based_on_file_input(&mut app, &buf, (1, 1));

        // Insert a char
        app.ui_state
            .file_input
            .as_mut()
            .expect("A file input has been opened, it can't be none")
            .handle(InputRequest::InsertChar('h'));

        acrp_based_on_file_input(&mut app, &buf, (2, 1));

        // Move cursor left
        app.ui_state
            .file_input
            .as_mut()
            .expect("A file input has been opened, it can't be none")
            .handle(InputRequest::GoToPrevChar);

        acrp_based_on_file_input(&mut app, &buf, (1, 1));

        // And right, then delete a char
        app.ui_state
            .file_input
            .as_mut()
            .expect("A file input has been opened, it can't be none")
            .handle(InputRequest::GoToNextChar);

        app.ui_state
            .file_input
            .as_mut()
            .expect("A file input has been opened, it can't be none")
            .handle(InputRequest::DeletePrevChar);

        acrp_based_on_file_input(&mut app, &buf, (1, 1));

        // Now some overflow
        let buf = Buffer::empty(Rect::new(0, 0, 4, 1));
        app.open_file_input("hello, world!", FileInputRole::GetOpenPath);
        // Does not reach (3, 1) because of the border
        acrp_based_on_file_input(&mut app, &buf, (2, 1))
    }

    #[test]
    fn test_app_handles_keybinds() {
        let config = r#"
            [keymaps]
            "ctrl+a" = "open_file"
            "#;
        let mut app = app_with_config(config);

        // A custom and a default keybind
        let open_file_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let close_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

        app.handle_key_event(open_file_event)
            .expect("Failed to handle key event");
        assert!(app.ui_state.file_input.is_some());
        assert_eq!(
            app.ui_state
                .file_input
                .as_ref()
                .expect("None case was handled")
                .role,
            FileInputRole::GetOpenPath
        );

        app.handle_key_event(close_event)
            .expect("Failed to handle key event");
        assert!(app.exit)
    }

    #[test]
    fn app_does_not_ask_for_save_path_if_there_is_one() {
        let mut app = app_with_file_contents("hello, world!");

        app.handle_operation(&Operation::SaveBufferToFile);

        assert!(app.ui_state.file_input.is_none());
    }

    #[test]
    fn app_asks_for_save_path_if_there_is_none() {
        let mut app = App::build_default();

        app.handle_operation(&Operation::SaveBufferToFile);

        assert!(app.ui_state.file_input.is_some());
        assert_eq!(
            app.ui_state.file_input.as_ref().unwrap().role,
            FileInputRole::GetSavePath
        );
    }
}
