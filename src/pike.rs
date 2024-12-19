use std::path::{Path, PathBuf};

use crate::config;
use crate::config::Config;
use scribe::buffer::Position;
use scribe::{Buffer, Workspace};

/// Backend of the app
#[allow(dead_code, unused_variables, unused_mut)]
pub struct Pike {
    workspace: Workspace,
    config: Config,
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
        let default_config_path = config::default_config_path();
        if config_file.is_none() {
            if let Ok(default_config_path) = default_config_path {
                if default_config_path.exists() {
                    config_file = Some(default_config_path.to_path_buf());
                }
            }
        }

        let mut workspace =
            Workspace::new(&cwd, None).map_err(|e| format!("Error creating workspace: {}", e))?;

        if let Some(cwf) = cwf {
            workspace
                .open_buffer(cwf.as_path())
                .map_err(|_| "Error opening file")?;
        }

        Ok(Pike {
            workspace,
            config: Config::from_file(config_file.as_deref())
                .map_err(|e| format!("Error loading config: {}", e))?,
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
            .move_to(Position { line, offset });

        Ok(())
    }

    /// Writes `text` to current buffer
    fn write_to_current_buffer(&mut self, text: &str) -> Result<(), String> {
        match &mut self.workspace.current_buffer {
            Some(buffer) => {
                buffer.insert(text);
                Ok(())
            }
            None => Err("Trying to write to a non-existent buffer".to_string()),
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

    pub fn current_buffer_path(&self) -> Option<&Path> {
        self.workspace.current_buffer_path()
    }

    /// Returns the filename of the current buffer or an empty string
    pub fn current_buffer_filename(&self) -> String {
        match self.current_buffer_path() {
            // TODO: check this goofy ahh chain
            Some(path) => path.file_name().unwrap().to_str().unwrap().to_string(),
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
    pub fn cursor_position(&self) -> Option<Position> {
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

    /// Move the cursor right if possible, else do nothing
    pub fn move_cursor_right(&mut self) {
        if let Some(buffer) = &mut self.workspace.current_buffer {
            buffer.cursor.move_right();
        }
    }

    /// Create a new empty buffer and set it as the current buffer
    fn new_buffer(&mut self) {
        todo!()
    }

    /// Switch to the previous buffer
    fn previous_buffer(&mut self) {
        todo!()
    }

    /// Switch to the next buffer
    fn next_buffer(&mut self) {
        todo!()
    }

    /// Search for a query in the current buffer and return
    /// the results in the form of a vec of offsets
    fn search_in_current_buffer(&mut self, query: &str) -> Vec<usize> {
        todo!()
    }

    /// Replace all occurences of query with replacement in the current buffer
    fn replace_in_current_buffer(&mut self, query: &str, replacement: &str) {
        todo!()
    }

    /// Save the current buffer to its file
    fn save_current_buffer(&mut self) -> Result<(), String> {
        match &mut self.workspace.current_buffer {
            Some(buffer) => {
                buffer.save().expect("Failed to save buffer");
                Ok(())
            }
            None => Err("Trying to save a non-existent buffer".to_string()),
        }
    }

    /// Undo the last change in the current buffer
    fn undo(&mut self) {
        todo!()
    }

    /// Redo the last change in the current buffer
    fn redo(&mut self) {
        todo!()
    }

    /// Returns the current working directory as a pathbuf
    fn cwd(&self) -> PathBuf {
        self.workspace.path.clone()
    }
}

#[cfg(test)]
mod pike_test {
    use std::{env, path::PathBuf};

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
    fn test_write_to_nonexisting_buffer() {
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        let result = pike.write_to_current_buffer("Hello, world!");

        assert_eq!(
            result,
            Err("Trying to write to a non-existent buffer".to_string())
        );
    }

    #[test]
    fn test_save_current_buffer() {
        let file = temp_file_with_contents("Hello, world!");
        let mut pike = tmp_pike_and_working_dir(None, None).0;

        pike.open_file(file.path(), 0, 0)
            .expect("Failed to open file");
        pike.save_current_buffer().expect("Failed to save buffer");

        let contents = std::fs::read_to_string(file.path()).expect("Failed to read file");
        assert_eq!(contents, "Hello, world!");
    }

    #[test]
    fn test_save_nonexisting_buffer() {
        let mut pike = tmp_pike_and_working_dir(None, None).0;
        let result = pike.save_current_buffer();

        assert_eq!(
            result,
            Err("Trying to save a non-existent buffer".to_string())
        );
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
    fn test_has_unsaved_changes_no_buffer() {
        let pike = tmp_pike_and_working_dir(None, None).0;

        assert!(!pike.has_unsaved_changes());
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
}
