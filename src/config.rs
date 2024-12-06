use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use toml::Table;

use crate::operations::Operation;
use std::collections::{HashMap, HashSet};

/// Represents a single shortcut consisting of a key and modifiers
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct KeyShortcut {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[allow(dead_code)]
impl KeyShortcut {
    fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyShortcut {
        KeyShortcut { code, modifiers }
    }

    /// Creates a new KeyShortcut from a crossterm::event::KeyEvent
    fn from_key_event(event: &KeyEvent) -> KeyShortcut {
        KeyShortcut::new(event.code, event.modifiers)
    }

    /// Creates a new KeyShortcut based on a string from a config file.
    /// String representation of the shortcut follows the VSCode notation,
    /// e.g. ctrl+shift+p, ctrl+alt+del
    fn from_string(s: &str) -> Result<KeyShortcut, String> {
        let elements_lowercase = s.split("+").map(|s| s.to_lowercase());
        let mut modifiers = KeyModifiers::empty();
        let mut code = KeyCode::Null;

        for element in elements_lowercase {
            if let Some(modifier) = key_modifier_from_string(&element) {
                modifiers |= modifier;
            } else {
                if code != KeyCode::Null {
                    return Err(String::from(
                        "More than one non-modifier keycode in keybind: {s}",
                    ));
                }
                code = keycode_from_string(&element)?;
            }
        }

        let shortcut = KeyShortcut::new(code, modifiers);
        if shortcut.is_empty() {
            return Err(String::from("No keycode found in keybind: {s}"));
        }
        Ok(shortcut)
    }

    /// Returns true if the shortcut is empty, i.e. no key or modifier is set
    fn is_empty(&self) -> bool {
        self.code == KeyCode::Null && self.modifiers == KeyModifiers::empty()
    }
}

/// Returns a KeyModifiers object from a string representation
/// or None if it does not match any.
fn key_modifier_from_string(s: &str) -> Option<KeyModifiers> {
    match s {
        "ctrl" => Some(KeyModifiers::CONTROL),
        "shift" => Some(KeyModifiers::SHIFT),
        "alt" => Some(KeyModifiers::ALT),
        _ => None,
    }
}

/// Returns a KeyCode from a string representation.
/// The input should be mappable to a valid keycode and not a modifier.
fn keycode_from_string(s: &str) -> Result<KeyCode, String> {
    let return_value = match s {
        "esc" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        other => {
            if other.chars().count() != 1 {
                return Err(String::from("Invalid keycode: {s}"));
            }
            KeyCode::Char(other.chars().next().unwrap())
        }
    };

    Ok(return_value)
}

/// Editor configuration
#[derive(Debug)]
pub struct Config {
    key_mappings: HashMap<KeyShortcut, Operation>,
}

#[allow(dead_code)]
impl Config {
    /// Creates a config instance based on toml string representation
    fn from_toml_representation(s: &str) -> Result<Config, String> {
        let mut return_value = Config::default();

        let parsed = s
            .parse::<Table>()
            .map_err(|e| format!("Error parsing configuration file: {e}"))?;

        if let Some(keymap_table) = parsed.get("keymaps").and_then(|keys| keys.as_table()) {
            let keymap_pairs = Config::keymap_pairs_from_toml_table(keymap_table)?;
            return_value.key_mappings.extend(keymap_pairs);
        }

        Ok(return_value)
    }

    fn keymap_pairs_from_toml_table(
        table: &Table,
    ) -> Result<Vec<(KeyShortcut, Operation)>, String> {
        let mut return_value = Vec::<(KeyShortcut, Operation)>::new();
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

            return_value.push((shortcut, op));
        }
        Ok(return_value)
    }

    /// Loads the configuration from the config file and returns it
    fn from_file(path: &str) -> Result<Config, String> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| format!("Error reading file: {e}"))?;
        Config::from_toml_representation(&contents)
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
                KeyShortcut::new(KeyCode::Tab, KeyModifiers::CONTROL),
                Operation::SwitchToNextBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Tab, KeyModifiers::SHIFT | KeyModifiers::CONTROL),
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
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
                Operation::SearchAndReplaceInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
                Operation::Undo,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('z'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
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
        ]);

        Config { key_mappings }
    }
}

#[cfg(test)]
mod key_shortcut_test {

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::KeyShortcut;

    #[test]
    fn from_event() {
        let events = [
            (KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())),
            (KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT)),
            (KeyEvent::new(
                KeyCode::Char('p'),
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            )),
            (KeyEvent::new(
                KeyCode::Char('y'),
                KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
            )),
            (KeyEvent::new(
                KeyCode::Null,
                KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
            )),
            (KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)),
            (KeyEvent::new(KeyCode::F(1), KeyModifiers::SHIFT)),
        ];

        let shortcuts = events.map(|event| KeyShortcut::from_key_event(&event));

        for (event, shortcut) in events.iter().zip(shortcuts.iter()) {
            assert_eq!(
                shortcut.code, event.code,
                "Keycode {:?} does not match {:?}",
                event.code, shortcut.code
            );
            assert_eq!(
                shortcut.modifiers, event.modifiers,
                "Modifier {:?} does not match {:?}",
                event.modifiers, shortcut.modifiers
            );
        }
    }

    #[test]
    fn from_string_valid_test_cases() {
        let strings_and_keymaps = vec![
            (
                "q",
                KeyShortcut::new(KeyCode::Char('q'), KeyModifiers::empty()),
            ),
            (
                "shift+s",
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
            ),
            (
                "ctrl+shift+Y",
                KeyShortcut::new(
                    KeyCode::Char('y'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
            ),
            (
                "y+shift+Ctrl",
                KeyShortcut::new(
                    KeyCode::Char('y'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
            ),
            (
                "Ctrl+Shift+Alt",
                KeyShortcut::new(
                    KeyCode::Null,
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
                ),
            ),
            (
                "esc+shift",
                KeyShortcut::new(KeyCode::Esc, KeyModifiers::SHIFT),
            ),
            (
                "shift+f1",
                KeyShortcut::new(KeyCode::F(1), KeyModifiers::SHIFT),
            ),
        ];

        let actual = strings_and_keymaps
            .iter()
            .map(|(s, _)| KeyShortcut::from_string(s).expect("Failed to parse valid keybind"))
            .collect::<Vec<KeyShortcut>>();

        for i in 0..actual.len() {
            assert_eq!(strings_and_keymaps[i].1, actual[i]);
        }
    }

    #[test]
    fn from_string_invalid_cases() {
        let invalid_strings = vec!["", "a+b", "abc", "ctrl+shift+alt+del+ctrl"];
        for s in invalid_strings {
            assert!(KeyShortcut::from_string(s).is_err());
        }
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
            "ctrl+shift+x" = "new_file"
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
                Operation::CreateNewFile,
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
                KeyShortcut::new(KeyCode::Tab, KeyModifiers::CONTROL),
                Operation::SwitchToNextBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Tab, KeyModifiers::SHIFT | KeyModifiers::CONTROL),
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
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
                Operation::SearchAndReplaceInCurrentBuffer,
            ),
            (
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                Operation::SaveBufferToFile,
            ),
            (
                KeyShortcut::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
                Operation::Undo,
            ),
            (
                KeyShortcut::new(
                    KeyCode::Char('z'),
                    KeyModifiers::SHIFT | KeyModifiers::CONTROL,
                ),
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

        let config = Config::from_file(temp_file.path().to_str().unwrap())
            .expect("Failed to parse config from file");

        assert_eq!(
            config
                .key_mappings
                .get(&KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::CONTROL,)),
            Some(&Operation::SaveBufferToFile)
        );
    }
}
