use std::path::{Path, PathBuf};

use crate::config;
use crate::config::Config;
use scribe::Workspace;

/// Backend of the app
pub struct Pike {
    workspace: Workspace,
    config: Config,
}

impl Pike {
    /// Create a new instance of Pike in a given directory
    fn new(path: &Path, file: Option<&Path>) -> Result<Pike, String> {
        todo!()
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
mod test {
    #[test]
    fn doesnt_fail() {
        assert!(true)
    }
}
