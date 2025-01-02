use crossterm::event::{KeyCode, KeyModifiers};
use toml::Table;

use crate::key_shortcut::KeyShortcut;
use crate::operations::Operation;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

/// Returns the default configuration path for pike regardless
/// of OS
pub fn default_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir();
    match config_dir {
        Some(mut path) => {
            path.push("pike.toml");
            Ok(path)
        }
        None => Err("Failed to get the configuration directory".to_string()),
    }
}

/// Editor configuration
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub key_mappings: HashMap<KeyShortcut, Operation>,
}

#[allow(dead_code)]
impl Config {
    /// Creates a config instance based on toml string representation
    pub fn from_toml_representation(s: &str) -> Result<Config, String> {
        let mut return_value = Config::default();

        let parsed = s
            .parse::<Table>()
            .map_err(|e| format!("Error parsing configuration file: {e}"))?;

        if let Some(keymap_table) = parsed.get("keymaps").and_then(|keys| keys.as_table()) {
            let keymap_pairs = Config::keymap_pairs_from_toml_table(keymap_table)?;

            // Reverse the key_mappings (switch KeyShortcut and Operation)
            let mut reversed_keymaps: HashMap<Operation, KeyShortcut> = return_value
                .key_mappings
                .iter()
                .map(|(sh, op)| (op.clone(), sh.clone()))
                .collect();

            // Extend the reversed keymap with new keymap pairs
            for (op, sh) in keymap_pairs {
                reversed_keymaps.insert(op, sh);
            }

            // Rebuild the key_mappings with reversed keys and operations
            return_value.key_mappings = reversed_keymaps
                .into_iter()
                .map(|(op, sh)| (sh, op))
                .collect();
        }

        Ok(return_value)
    }

    /// Loads the configuration from the config file and returns it
    pub fn from_file(path: Option<&Path>) -> Result<Config, String> {
        match path {
            Some(path) => {
                let contents = std::fs::read_to_string(path)
                    .map_err(|e| format!("Error reading file: {e}"))?;
                Config::from_toml_representation(&contents)
            }
            None => Ok(Config::default()),
        }
    }

    /// Creates a vector of pairs (shortcut, operation) to
    /// be inserted into the config's keymap section
    /// over the default configuration
    fn keymap_pairs_from_toml_table(
        table: &Table,
    ) -> Result<Vec<(Operation, KeyShortcut)>, String> {
        let mut return_value = Vec::<(Operation, KeyShortcut)>::new();
        let mut seen_shortcuts = HashSet::<KeyShortcut>::new();
        let mut seen_operations = HashSet::<Operation>::new();

        for (shortcut, op) in table {
            let shortcut = KeyShortcut::from_string(shortcut)?;
            let op = Operation::from_string(op.as_str().unwrap())?;

            if !seen_shortcuts.insert(shortcut.clone()) {
                return Err(format!("Duplicate keybinding found: {:?}", shortcut));
            }

            // Check for duplicate operations
            if !seen_operations.insert(op.clone()) {
                return Err(format!("Duplicate keymap operation found: {:?}", op));
            }

            return_value.push((op, shortcut));
        }
        Ok(return_value)
    }
}

impl Default for Config {
    fn default() -> Config {
        let key_mappings = HashMap::<KeyShortcut, Operation>::from([
            (
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                Operation::SaveBufferToFile,
            ),
            (
                KeyShortcut::new(KeyCode::Char('o'), KeyModifiers::CONTROL),
                Operation::OpenFile,
            ),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                Operation::CreateNewBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
                Operation::SwitchToNextBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
                Operation::SwitchToPreviousBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
                Operation::OpenBufferPicker,
            ),
            (
                KeyShortcut::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
                Operation::SearchInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
                Operation::SearchAndReplaceInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
                Operation::Undo,
            ),
            (
                KeyShortcut::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
                Operation::Redo,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('f'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
                Operation::FindFilesInCWD,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('p'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
                Operation::FindTextInCWD,
            ),
            (
                KeyShortcut::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
                Operation::Quit,
            ),
        ]);

        Config { key_mappings }
    }
}

#[cfg(test)]
mod config_test {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::operations::Operation;

    use super::{Config, KeyShortcut};

    #[test]
    fn from_toml_keymap_section_valid_case() {
        let keymap_section = r#"
            [keymaps]
            "ctrl+shift+x" = "open_file"
            "#;

        let actual = Config::from_toml_representation(keymap_section)
            .expect("Failed to parse a valid keymap section")
            .key_mappings;
        let expected = HashMap::<KeyShortcut, Operation>::from_iter(vec![
            (
                KeyShortcut::new(
                    KeyCode::Char('x'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
                Operation::OpenFile,
            ),
            (
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                Operation::SaveBufferToFile,
            ),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                Operation::CreateNewBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
                Operation::SwitchToNextBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
                Operation::SwitchToPreviousBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
                Operation::OpenBufferPicker,
            ),
            (
                KeyShortcut::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
                Operation::SearchInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
                Operation::SearchAndReplaceInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
                Operation::Undo,
            ),
            (
                KeyShortcut::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
                Operation::Redo,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('f'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
                Operation::FindFilesInCWD,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('p'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
                Operation::FindTextInCWD,
            ),
            (
                KeyShortcut::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
                Operation::Quit,
            ),
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_toml_representation_keymap_section_duplicates() {
        let representations = [
            r#"
                [keymaps]
                "ctrl+s" = "open_file"
                "ctrl+y" = "open_file"
                "#,
            r#"
                [keymaps]
                "ctrl+s" = "save"
                "ctrl+s" = "open_file"
                "#,
        ];

        for s in representations {
            assert!(
                Config::from_toml_representation(s).is_err(),
                "Failed for: {s}"
            );
        }
    }

    #[test]
    fn from_toml_representation_invalid_keymap_section() {
        let invalid_representations = [
            "keymaps = {invalid}",
            r#"
                [keymaps]
                ctrl+s="nonexisting_action"
                "#,
        ];

        for s in invalid_representations {
            assert!(
                Config::from_toml_representation(s).is_err(),
                "Failed for: {s}"
            );
        }
    }

    #[test]
    fn test_from_file_valid_case() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let toml_content = r#"
        [keymaps]
        "ctrl+s" = "save"
    "#;
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write to temp file");

        let config =
            Config::from_file(Some(temp_file.path())).expect("Failed to parse config from file");

        assert_eq!(
            config
                .key_mappings
                .get(&KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::CONTROL,)),
            Some(&Operation::SaveBufferToFile)
        );
    }

    #[test]
    fn test_from_file_no_path() {
        let config = Config::from_file(None).expect("Failed to parse config from file");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_default_config_path() {
        let expected = dirs::config_dir().unwrap().join("pike.toml");
        let actual = super::default_config_path().expect("Failed to get default config path");
        assert_eq!(expected, actual);
    }
}
