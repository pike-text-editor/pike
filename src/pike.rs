use std::path::{self, Path, PathBuf};

use crate::config;
use crate::config::Config;
use scribe::Workspace;

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
        // TODO: figure out how to test this
        if config_file.is_none() && path::Path::new(config::DEFAULT_CONFIG_PATH).exists() {
            config_file = Some(PathBuf::from(config::DEFAULT_CONFIG_PATH));
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
    /// and set the cursor to the offset
    fn open_file(&mut self, path: &Path, offset: u32) {
        todo!()
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
    fn save_current_buffer(&self) {
        todo!()
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

    use crate::config::Config;
    use std::io::Write;
    use tempfile::NamedTempFile;

    use super::Pike;

    fn temp_file_with_contents(contents: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(contents.as_bytes())
            .expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_build_minimal_args() {
        let dir = env::temp_dir();
        let cwd = PathBuf::from(dir.as_path());
        let pike = Pike::build(cwd.clone(), None, None).expect("Failed to build Pike");

        assert_eq!(pike.workspace.path, cwd);
        assert!(pike.workspace.current_buffer.is_none());
        assert!(pike.config == Config::default());
    }

    #[test]
    fn test_build_max_args() {
        let dir = env::temp_dir();
        let config_content = r#"
            [keymaps]
            "ctrl+a" = "save"
        "#;

        let config_file = temp_file_with_contents(config_content);
        let working_file = temp_file_with_contents("hello, world!");

        let cwd = PathBuf::from(dir.as_path());
        let cwf = Some(working_file.path().to_path_buf());
        let config_path = Some(config_file.path().to_path_buf());

        let pike = Pike::build(cwd.clone(), cwf, config_path).expect("Failed to build Pike");
        assert_eq!(pike.workspace.path, cwd);
        assert_eq!(
            pike.workspace
                .current_buffer
                .expect("Current buffer shouldn't be empty when set")
                .data(),
            "hello, world!"
        );
        let expected_config =
            Config::from_file(Some(config_file.path())).expect("Failed to load config from file");
        assert_eq!(pike.config, expected_config);
    }
}
