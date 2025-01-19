use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::config;
use crate::config::Config;
use crate::key_shortcut::KeyShortcut;
use crate::operations::Operation;
use scribe::buffer::Position as BufferPosition;
use scribe::{Buffer, Workspace};
use unicode_segmentation::UnicodeSegmentation;

/// Cursor history
#[derive(Default)]
struct CursorHistory {
    undo_stack: Vec<BufferPosition>,
    redo_stack: Vec<BufferPosition>,
}

impl CursorHistory {
    /// Record a new cursor position on the undo stack.
    fn record_undo_position(&mut self, pos: BufferPosition) {
        self.undo_stack.push(pos);
        // Once you record a new position, clear the redo stack.
        self.redo_stack.clear();
    }
}

#[derive(Default)]
pub struct Highlight {
    pub start: BufferPosition,
    pub length: usize,
    pub is_selected: bool,
}

/// Backend of the app
#[allow(dead_code, unused_variables, unused_mut)]
pub struct Pike {
    workspace: Workspace,
    config: Config,
    cursor_history: CursorHistory,
}

#[allow(dead_code, unused_variables, unused_mut)]
impl Pike {
    /// Create a new instance of Pike in a given directory
    pub fn build(
        cwd: PathBuf,
        cwf: Option<PathBuf>,
        mut config_file: Option<PathBuf>,
    ) -> Result<Pike, String> {
        // If no config path is provided, check if the default config file exists
        if config_file.is_none() {
            let default_config_file_path = config::default_config_file_path();
            if let Ok(default_config_path) = default_config_file_path {
                if default_config_path.exists() {
                    config_file = Some(default_config_path.to_path_buf());
                }
            }
        }

        let mut workspace =
            Workspace::new(&cwd, None).map_err(|e| format!("Error creating workspace: {}", e))?;

        if let Some(cwf) = cwf {
            // Check if file exits, if not, create it
            if !cwf.exists() {
                if let Some(parent) = cwf.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                File::create(&cwf).map_err(|e| format!("Failed to create file: {}", e))?;
            }
            // Open the given file
            workspace
                .open_buffer(cwf.as_path())
                .map_err(|_| "Error opening file")?;
        }
        Ok(Pike {
            workspace,
            config: Config::from_file(config_file.as_deref())
                .map_err(|e| format!("Error loading config: {}", e))?,
            cursor_history: CursorHistory::default(),
        })
    }

    /// Open a file, move its contents into the current buffer
    /// and set the cursor to the offset. If the offset is out of bounds,
    /// the cursor will remain at the start of the file.
    pub fn open_file(&mut self, path: &Path, line: usize, offset: usize) -> Result<(), String> {
        self.workspace
            .open_buffer(path)
            .map_err(|_| "Error opening file".to_string())?;

        self.workspace
            .current_buffer
            .as_mut()
            .expect("Scribe's open_buffer should set a buffer")
            .cursor
            .move_to(BufferPosition { line, offset });

        Ok(())
    }

    /// Create a file if if does not exists and open it
    pub fn create_and_open_file(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            File::create(path).map_err(|e| {
                format!(
                    "Failed to create file: {} ({})",
                    path.to_str()
                        .expect("A path to file has to be valid unicode"),
                    e
                )
            })?;
        }
        self.open_file(path, 0, 0)?;
        Ok(())
    }

    /// Writes `text` to current buffer
    pub fn write_to_current_buffer(&mut self, text: &str) -> Result<(), String> {
        match &mut self.workspace.current_buffer {
            Some(buffer) => {
                // Remember the cursor position before inserting
                let start_position = buffer.cursor.position;

                self.cursor_history.record_undo_position(start_position);

                buffer.insert(text);

                // Calculate how many lines the inserted text spans
                let lines: Vec<&str> = text.split('\n').collect();
                let line_count = lines.len() - 1;

                // On the final line, check where it ends
                let last_line_len = lines.last().map_or(0, |l| l.chars().count());

                // If no newlines were inserted, just advance on the same line
                // Otherwise, move down line_count lines, then set offset to the length of the last line
                let new_line = start_position.line + line_count;
                let new_offset = if line_count == 0 {
                    start_position.offset + last_line_len
                } else {
                    last_line_len
                };

                buffer.cursor.move_to(scribe::buffer::Position {
                    line: new_line,
                    offset: new_offset,
                });

                Ok(())
            }
            None => Err("Trying to write to a non-existent buffer".to_string()),
        }
    }

    /// Deletes a characted and moves the cursor left
    pub fn delete_character_from_current_buffer(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            let pos = buffer.cursor.position;

            self.cursor_history.record_undo_position(pos);

            let data = buffer.data();

            let lines: Vec<&str> = data.split('\n').collect();

            let current_line_length = lines.get(pos.line).map_or(0, |line| line.len());

            if pos.offset == 0 && pos.line > 0 {
                buffer.cursor.move_up();

                let new_offset = {
                    let new_pos = buffer.cursor.position.line;
                    lines.get(new_pos).map_or(0, |line| line.len())
                };

                buffer.cursor.move_to(scribe::buffer::Position {
                    line: buffer.cursor.position.line,
                    offset: new_offset,
                });

                // Delete here so it removes the newline
                buffer.delete();
            } else if pos.offset > 0 {
                buffer.cursor.move_left();
                buffer.delete();
            }
        }
    }

    /// Returns the contents of the currently opened buffer or
    /// an empty string if none is open
    pub fn current_buffer_contents(&self) -> String {
        match self.current_buffer().as_ref() {
            Some(buffer) => buffer.data(),
            None => String::from(""),
        }
    }

    /// Returns an absolute path to the current buffer or None
    pub fn current_buffer_path(&self) -> Option<PathBuf> {
        self.workspace
            .current_buffer_path()
            .map(|buf| self.workspace.path.join(buf))
    }

    /// Returns the filename of the current buffer or an empty string
    pub fn current_buffer_filename(&self) -> String {
        match self.current_buffer_path() {
            Some(path) => path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(|s| s.to_string())
                .expect("Failed to convert filename to string"),
            None => String::from(""),
        }
    }

    /// Returns whether the current buffer has unsaved changes or
    /// false if it's empty
    pub fn has_unsaved_changes(&self) -> bool {
        match &self.current_buffer() {
            Some(buffer) => buffer.modified(),
            None => false,
        }
    }

    /// Returns the position of the cursor in the current buffer
    /// or None if there isn't one
    pub fn cursor_position(&self) -> Option<BufferPosition> {
        self.workspace
            .current_buffer
            .as_ref()
            .map(|buffer| buffer.cursor.position)
    }

    /// Getter for the current buffer
    pub fn current_buffer(&self) -> Option<&Buffer> {
        self.workspace.current_buffer.as_ref()
    }

    /// Move the cursor up if possible, else do nothing
    pub fn move_cursor_up(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_up();
        }
    }

    /// Move the cursor down if possible, else do nothing
    pub fn move_cursor_down(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_down();
        }
    }

    /// Move the cursor left if possible, else do nothing
    pub fn move_cursor_left(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_left();
        }
    }

    /// Move the cursor to the start of line if possible, else do nothing
    pub fn move_cursor_to_start_of_line(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_to_start_of_line();
        }
    }

    /// Move the cursor to the endf of line if possible, else do nothing
    pub fn move_cursor_to_end_of_line(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_to_end_of_line();
        }
    }

    /// Move the cursor left by one word if possible, else do nothing
    pub fn move_cursor_left_by_word(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            let pos = buffer.cursor.position;

            // Split the entire buffer by lines.
            let data = buffer.data();
            let lines: Vec<&str> = data.lines().collect();
            if lines.is_empty() {
                return; // nothing to move around
            }

            // If we're already at the very start of the file, do nothing.
            if pos.line == 0 && pos.offset == 0 {
                return;
            }

            // Determine the new line and offset (which we will move to).
            // If the offset is 0, we need to move up one line.
            let mut new_line = pos.line;
            let mut new_offset = pos.offset;

            if new_offset == 0 {
                // Move up a line, set offset to the end of that line.
                new_line -= 1;
                let prev_line_str = lines[new_line];
                new_offset = prev_line_str.graphemes(true).count();
            }

            // Now we’re guaranteed to have new_offset > 0
            // (because if it was zero, we just moved up a line).
            let line_str = lines[new_line];
            let graphemes: Vec<&str> = line_str.graphemes(true).collect();

            // Skip trailing whitespace leftwards
            while new_offset > 0 && graphemes[new_offset - 1].trim().is_empty() {
                new_offset -= 1;
            }

            // Skip over the word leftwards
            while new_offset > 0 && !graphemes[new_offset - 1].trim().is_empty() {
                new_offset -= 1;
            }

            buffer.cursor.move_to(BufferPosition {
                line: new_line,
                offset: new_offset,
            });
        }
    }

    /// Move the cursor right by one word if possible, else do nothing
    pub fn move_cursor_right_by_word(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            let pos = buffer.cursor.position;

            // Split the entire buffer by lines.
            let data = buffer.data();
            let lines: Vec<&str> = data.lines().collect();

            // If there's nothing in the buffer, no movement.
            if lines.is_empty() {
                return;
            }

            let current_line_len = lines[pos.line].graphemes(true).count();

            // Check if we are at the very end of the file.
            // i.e., at the last line and at the line's end.
            if pos.line == lines.len() - 1 && pos.offset == current_line_len {
                return;
            }

            let (mut new_line, mut new_offset) = (pos.line, pos.offset);

            // If we are at the end of the current line, move down to the next line (offset = 0).
            if new_offset >= current_line_len {
                new_line += 1;
                new_offset = 0;
            } else {
                // Otherwise, we are somewhere in the middle of the line.
                let line_str = lines[new_line];
                let graphemes: Vec<&str> = line_str.graphemes(true).collect();
                let line_len = graphemes.len();

                // Skip over any whitespace to the right
                while new_offset < line_len && graphemes[new_offset].trim().is_empty() {
                    new_offset += 1;
                }

                // Skip over the word to the right
                while new_offset < line_len && !graphemes[new_offset].trim().is_empty() {
                    new_offset += 1;
                }
            }

            buffer.cursor.move_to(BufferPosition {
                line: new_line,
                offset: new_offset,
            });
        }
    }

    /// Move the cursor right if possible, else do nothing
    pub fn move_cursor_right(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_right();
        }
    }

    /// Move the cursor to a specific position
    pub fn move_cursor_to(&mut self, pos: BufferPosition) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_to(pos);
        }
    }

    /// Returns the length of the current line
    pub fn current_line_length(&self) -> usize {
        let current_line_number = self.cursor_position().map_or(0, |pos| pos.line);
        match self
            .current_buffer_contents()
            .lines()
            .nth(current_line_number)
        {
            Some(line) => line.len(),
            None => 0,
        }
    }

    /// Create a new empty buffer not bound to a path and set it as the current buffer
    pub fn open_new_buffer(&mut self) {
        let buf = Buffer::new();
        self.workspace.add_buffer(buf);
    }

    /// Switch to the previous buffer
    pub fn previous_buffer(&mut self) {
        self.workspace.previous_buffer();
        // Clear the cursor history when switching buffers
        self.cursor_history = CursorHistory::default();
    }

    /// Switch to the next buffer
    pub fn next_buffer(&mut self) {
        self.workspace.next_buffer();
        // Clear the cursor history when switching buffers
        self.cursor_history = CursorHistory::default();
    }

    /// Search for a query in the current buffer and return
    /// the results in the form of a vec of offsets
    pub fn search_in_current_buffer(&mut self, query: &str) -> Result<Vec<Highlight>, String> {
        if let Some(buf) = self.workspace.current_buffer.as_mut() {
            let results = buf
                .search(query)
                .into_iter()
                .map(|pos| Highlight {
                    start: pos,
                    length: query.len(),
                    is_selected: false,
                })
                .collect();
            Ok(results)
        } else {
            Err("No buffer is currently open".to_string())
        }
    }

    /// Save the current buffer to its file
    pub fn save_current_buffer(&mut self) -> Result<(), String> {
        match &mut self.workspace.current_buffer {
            Some(buffer) => {
                buffer.save().expect("Failed to save buffer");

                Ok(())
            }
            None => Err("Trying to save a non-existent buffer".to_string()),
        }
    }

    /// Check if the current buffer has been modified
    pub fn is_current_buffer_modified(&self) -> bool {
        match self.current_buffer() {
            Some(buffer) => buffer.modified(),
            None => false,
        }
    }

    /// Undo the last change in the current buffer and adjust the cursor position
    pub fn undo(&mut self) {
        if let Some(buf) = self.workspace.current_buffer.as_mut() {
            // If there's a recorded position, pop it off
            if let Some(prev_pos) = self.cursor_history.undo_stack.pop() {
                // Push the current cursor position onto redo stack.
                let current_pos = buf.cursor.position;
                self.cursor_history.redo_stack.push(current_pos);

                buf.undo();

                // Move cursor to the old position
                buf.cursor.move_to(prev_pos);
            }
        }
    }

    /// Redo the last change in the current buffer and adjust the cursor position
    pub fn redo(&mut self) {
        if let Some(buf) = self.workspace.current_buffer.as_mut() {
            // If there's a position we previously popped off, pop it from redo
            if let Some(pos) = self.cursor_history.redo_stack.pop() {
                // Push the current cursor position onto undo stack
                // so we can jump back if we undo the redo.
                let current_pos = buf.cursor.position;
                self.cursor_history.undo_stack.push(current_pos);

                buf.redo();

                // Move the cursor to the position after redo
                buf.cursor.move_to(pos);
            }
        }
    }

    /// Returns the current working directory as a pathbuf
    fn cwd(&self) -> PathBuf {
        self.workspace.path.clone()
    }

    /// Gets an operation corresponding to a key shortcut
    pub fn get_keymap(&self, mapping: &KeyShortcut) -> Option<&Operation> {
        self.config.key_mappings.get(mapping)
    }

    /// Sets a path for the current buffer
    pub fn bind_current_buffer_to_path(&mut self, path: PathBuf) {
        if let Some(buf) = self.workspace.current_buffer.as_mut() {
            buf.path = Some(path);
        }
    }
}

#[cfg(test)]
mod pike_test {
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    use crate::{config::Config, test_util::temp_file_with_contents};
    use scribe::buffer::Position;

    use super::Pike;

    /// Setup before a test, creates an instance of pike in
    /// a temporary directory and returns them. Optionally takes
    /// in the string contents to be injected into its config and
    /// current working files.
    fn tmp_pike_and_working_dir(
        config_content: Option<&str>,
        cwf_content: Option<&str>,
    ) -> (Pike, PathBuf) {
        let dir = env::temp_dir();
        let cwd = PathBuf::from(dir.as_path())
            .canonicalize()
            .expect("Failed to canonicalize path");
        let cwf = cwf_content.map(temp_file_with_contents);
        let config_file = config_content.map(temp_file_with_contents);
        let cwf_path = cwf.as_ref().map(|f| f.path().to_path_buf());
        let config_path = config_file.as_ref().map(|f| f.path().to_path_buf());

        (
            Pike::build(cwd.clone(), cwf_path, config_path).expect("Failed to build Pike"),
            cwd,
        )
    }

    /// Canonicalizes two paths and asserts their equality
    fn assert_paths(path1: &Path, path2: &Path) {
        assert_eq!(
            path1.canonicalize().expect("Failed to canonicalize path"),
            path2.canonicalize().expect("Failed to canonicalize path")
        );
    }

    #[test]
    fn test_build_minimal_args() {
        let (pike, cwd) = tmp_pike_and_working_dir(None, None);

        assert_eq!(pike.workspace.path, cwd);
        assert!(pike.current_buffer().is_none());
        assert!(pike.config == Config::default());
    }

    #[test]
    fn test_build_max_args() {
        let config_content = r#"
            [keymaps]
            "ctrl+a" = "save"
        "#;
        let file_content = "hello, world!";
        let (pike, cwd) = tmp_pike_and_working_dir(Some(config_content), Some(file_content));

        assert_eq!(pike.workspace.path, cwd);
        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Current buffer shouldn't be empty when set")
                .data(),
            "hello, world!"
        );
        let expected_config =
            Config::from_toml_representation(config_content).expect("Failed to parse config");
        assert_eq!(pike.config, expected_config);
    }

    #[test]
    fn test_open_zero_offset() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        assert_eq!(
            pike.workspace
                .current_buffer_path()
                .expect("Buffer should be set after opening a file")
                .file_name()
                .expect("File should have a name"),
            file.path().file_name().expect("File should have a name")
        );

        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Buffer should be set after opening a file")
                .data(),
            "Hello, world!"
        );
    }

    #[test]
    fn test_open_file_non_zero_offset() {
        let file_contents = r#"
            Hello,
            World
            "#;
        let file = temp_file_with_contents(file_contents);
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 1, 2)
            .expect("Could not open file");

        assert_eq!(
            pike.workspace
                .current_buffer_path()
                .expect("Buffer should be set after opening a file")
                .file_name()
                .expect("File should have a name"),
            file.path().file_name().expect("File should have a name")
        );

        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Should have an open buffer!")
                .cursor
                .position,
            Position { line: 1, offset: 2 }
        );
    }

    #[test]
    fn test_open_file_out_of_bounds_offset() {
        let file_contents = r#"
            Hello,
            World
            "#;
        let file = temp_file_with_contents(file_contents);
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 2, 100)
            .expect("Could not open file");

        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Should have an open buffer!")
                .cursor
                .position,
            Position { line: 0, offset: 0 }
        );
    }

    #[test]
    fn test_write_to_buffer() {
        let mut pike = tmp_pike_and_working_dir(None, Some("")).0;
        pike.write_to_current_buffer("Hello, world!")
            .expect("Failed to write to buffer");

        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Should have an open buffer!")
                .data(),
            "Hello, world!"
        );
    }

    #[test]
    fn test_write_to_unbound_buffer() {
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_new_buffer();
        let result = pike.write_to_current_buffer("Hello, world!");
        assert!(result.is_ok());
        assert_eq!(pike.current_buffer_contents(), "Hello, world!");
        pike.write_to_current_buffer(" Its me!")
            .expect("Failed to write to buffer");
        assert_eq!(pike.current_buffer_contents(), "Hello, world! Its me!");
    }

    #[test]
    fn test_save_current_buffer() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");
        pike.save_current_buffer().expect("Failed to save buffer");

        let contents = fs::read_to_string(file.path()).expect("Failed to read file");
        assert_eq!(contents, "Hello, world!");
    }

    #[test]
    #[should_panic]
    fn test_save_buffer_no_path() {
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_new_buffer();
        // This situation should not happen as it's handled in the UI, so a panic here
        // is expected
        let _ = pike.save_current_buffer();
    }

    #[test]
    fn test_current_buffer_contents_has_buffer() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        assert_eq!(pike.current_buffer_contents(), "Hello, world!");
    }

    #[test]
    fn test_current_buffer_contents_no_buffer() {
        let pike = tmp_pike_and_working_dir(None, None).0;

        assert_eq!(pike.current_buffer_contents(), "");
    }

    #[test]
    fn test_current_buffer_fname_has_buffer() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        assert_eq!(
            pike.current_buffer_filename(),
            file.path().file_name().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn test_current_buffer_fname_no_buffer() {
        let pike = tmp_pike_and_working_dir(None, None).0;

        assert_eq!(pike.current_buffer_filename(), "");
    }

    #[test]
    fn test_has_unsaved_changes_has_changes() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");
        pike.write_to_current_buffer("belo")
            .expect("Failed to write to file");

        assert!(pike.has_unsaved_changes());
    }

    #[test]
    fn test_has_unsaved_changes_no_changes() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        assert!(!pike.has_unsaved_changes());
    }

    #[test]
    fn test_has_unsaved_changes_new_buffer() {
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        pike.open_new_buffer();
        assert!(pike.has_unsaved_changes());
    }

    /// When moving down to a shorter line, the
    /// cursor position should be clamped to its length
    #[test]
    fn test_move_cursor_down_shorter_line() {
        let contents = r#"Hello!

        This is a test."#;
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));
        for _ in 0..5 {
            pike.move_cursor_right();
        }

        pike.move_cursor_down();
        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 1, offset: 0 })
        );
    }

    /// The cursor should not move out of the bounds of the current
    /// buffer
    #[test]
    fn test_move_cursor_out_of_bounds() {
        let contents = "a";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_right();
        assert_eq!(
            pike.cursor_position(),
            // This makes sense, since inserting does not move the cursor right
            Some(Position { line: 0, offset: 1 })
        );

        // Two times to the left to test for going too far to the left
        pike.move_cursor_left();
        pike.move_cursor_left();
        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );

        pike.move_cursor_down();
        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );

        pike.move_cursor_up();
        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );
    }

    #[test]
    fn test_move_cursor_left_by_word() {
        let contents = "aaa aaa";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_to(Position { line: 0, offset: 4 });

        pike.move_cursor_left_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );
    }

    #[test]
    fn test_move_cursor_right_by_word() {
        let contents = "aaa aaa";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_to(Position { line: 0, offset: 0 });

        pike.move_cursor_right_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 3 })
        );
    }

    #[test]
    fn test_move_cursor_left_by_word_with_unicode() {
        let contents = "aaa ę aaa";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_to(Position { line: 0, offset: 6 });

        pike.move_cursor_left_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 4 })
        );

        pike.move_cursor_left_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );
    }

    #[test]
    fn test_move_cursor_right_by_word_with_unicode() {
        let contents = "aaa ę aaa";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_to(Position { line: 0, offset: 0 });

        pike.move_cursor_right_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 3 })
        );

        pike.move_cursor_right_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 5 })
        );
    }

    #[test]
    fn test_move_cursor_right_and_left_with_combining_unicode() {
        let contents = "ęęę ęęę";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents));

        pike.move_cursor_to(Position { line: 0, offset: 0 });

        pike.move_cursor_right_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 3 })
        );

        pike.move_cursor_right_by_word();
        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 7 })
        );

        pike.move_cursor_left_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 4 })
        );

        pike.move_cursor_left_by_word();

        assert_eq!(
            pike.cursor_position(),
            Some(Position { line: 0, offset: 0 })
        );
    }

    #[test]
    fn test_current_line_length_buffer_exists() {
        let contents = ["Hello!", ""].join("\n");
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(contents.as_str()));

        assert_eq!(pike.current_line_length(), 6);

        pike.move_cursor_down();
        assert_eq!(pike.current_line_length(), 0);
    }

    #[test]
    fn test_current_line_length_no_buffer() {
        let pike = tmp_pike_and_working_dir(None, None).0;

        assert_eq!(pike.current_line_length(), 0);
    }

    #[test]
    fn test_create_and_open_file_doesnt_exist() {
        let (mut pike, cwd) = tmp_pike_and_working_dir(None, None);
        let file_path = cwd.join("test.txt");

        pike.create_and_open_file(&file_path)
            .expect("Failed to create and open file");

        assert_paths(
            &pike
                .current_buffer_path()
                .expect("Buffer should be set after opening a file"),
            &file_path,
        );
    }

    #[test]
    fn test_create_and_open_file_nested() {
        let (mut pike, cwd) = tmp_pike_and_working_dir(None, None);
        let file_path = cwd.join("nested").join("test.txt");

        pike.create_and_open_file(&file_path)
            .expect("Failed to create and open file");

        assert_paths(
            &pike
                .current_buffer_path()
                .expect("Buffer should be set after opening a file"),
            &file_path,
        );
    }

    #[test]
    fn test_create_and_open_file_exists() {
        let file = temp_file_with_contents("Hello, world!");
        let (mut pike, _) = tmp_pike_and_working_dir(None, None);

        pike.create_and_open_file(file.path())
            .expect("Failed to create and open file");

        assert_paths(
            &pike
                .current_buffer_path()
                .expect("Buffer should be set after opening a file"),
            file.path(),
        );
    }

    #[test]
    fn test_open_new_buffer() {
        let file = temp_file_with_contents("Hello, world!");
        let (mut pike, _) = tmp_pike_and_working_dir(None, None);

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");
        assert_eq!(pike.workspace.buffer_paths().len(), 1);

        // Should be empty with no path
        pike.open_new_buffer();
        assert_eq!(pike.current_buffer_contents(), "");
        assert!(pike
            .current_buffer()
            .expect("A buffer should be open")
            .path
            .is_none());
        assert_eq!(pike.workspace.buffer_paths().len(), 2);
    }

    #[test]
    fn test_bind_current_buffer_to_path() {
        let file_contents = "Hello, world!";
        let (mut pike, dir) = tmp_pike_and_working_dir(None, None);
        assert!(pike.current_buffer_path().is_none());
        pike.open_new_buffer();
        pike.write_to_current_buffer(file_contents)
            .expect("Failed to write to current buffer");

        let file_path = dir.join(Path::new("new_file.txt"));
        pike.bind_current_buffer_to_path(file_path.clone());

        assert!(pike.save_current_buffer().is_ok());

        let contents_from_file =
            fs::read_to_string(file_path).expect("std::fs failed to read from file");
        assert_eq!(file_contents, contents_from_file)
    }

    #[test]
    fn test_moving_cursor_when_inserting_many_characters() {
        let file = temp_file_with_contents("Hello, world!");
        let (mut pike, _) = tmp_pike_and_working_dir(None, None);

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        pike.move_cursor_to(Position {
            line: 0,
            offset: 13,
        });
        pike.write_to_current_buffer(" Its me!")
            .expect("Failed to write to buffer");

        assert_eq!(pike.current_buffer_contents(), "Hello, world! Its me!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 21
            })
        );
    }

    #[test]
    fn test_undo() {
        let file = temp_file_with_contents("Hello, world!");
        let (mut pike, _) = tmp_pike_and_working_dir(None, None);

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        pike.move_cursor_to(Position {
            line: 0,
            offset: 13,
        });
        pike.write_to_current_buffer("!")
            .expect("Failed to write to buffer");

        assert_eq!(pike.current_buffer_contents(), "Hello, world!!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 14
            })
        );

        pike.undo();
        assert_eq!(pike.current_buffer_contents(), "Hello, world!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 13
            })
        );
    }

    #[test]
    fn test_redo() {
        let file = temp_file_with_contents("Hello, world!");
        let (mut pike, _) = tmp_pike_and_working_dir(None, None);

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");

        pike.move_cursor_to(Position {
            line: 0,
            offset: 13,
        });
        pike.write_to_current_buffer("!")
            .expect("Failed to write to buffer");

        assert_eq!(pike.current_buffer_contents(), "Hello, world!!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 14
            })
        );

        pike.undo();
        assert_eq!(pike.current_buffer_contents(), "Hello, world!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 13
            })
        );

        pike.redo();
        assert_eq!(pike.current_buffer_contents(), "Hello, world!!");
        assert_eq!(
            pike.cursor_position(),
            Some(Position {
                line: 0,
                offset: 14
            })
        );
    }

    #[test]
    fn test_search_in_current_buffer() {
        let file_contents = "Hello, world!";
        let (mut pike, _) = tmp_pike_and_working_dir(None, Some(file_contents));

        let results = pike
            .search_in_current_buffer("world")
            .expect("No buffer is currently open");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].start, Position { line: 0, offset: 7 });
        assert_eq!(results[0].length, 5);
    }
}
