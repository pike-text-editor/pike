use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::operations::Operation;
use std::collections::HashMap;

/// Represents a single shortcut consisting of a key and modifiers
#[allow(dead_code, unused_variables, unused_mut)]
#[derive(Debug)]
struct KeyShortcut {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[allow(dead_code, unused_variables, unused_mut)]
impl KeyShortcut {
    /// Creates a new KeyShortcut from a crossterm::event::KeyEvent
    fn from_key_event(event: &KeyEvent) -> KeyShortcut {
        KeyShortcut {
            code: event.code,
            modifiers: event.modifiers,
        }
    }

    /// Creates a new KeyShortcut based on a string from a config file
    /// TODO: decide which syntax it has to follow and document it here
    fn from_string(s: &str) -> KeyShortcut {
        todo!()
    }
}

/// Editor configuration
#[allow(dead_code, unused_variables, unused_mut)]
#[derive(Debug)]
pub struct Config {
    key_mappings: HashMap<KeyShortcut, Operation>,
}

#[allow(dead_code, unused_variables, unused_mut)]
/// Loads the configuration from the config file and returns it
pub fn load_config() -> Config {
    // TODO: convert to hashmap using toml and parse keybinds
    todo!()
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
            (KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::SHIFT,
            )),
            (KeyEvent::new(
                KeyCode::F(1),
                KeyModifiers::SHIFT,
            ))
        ];

        let shortcuts = events.map(|event| KeyShortcut::from_key_event(&event));

        for (event, shortcut) in events.iter().zip(shortcuts.iter()) {
            assert_eq!(shortcut.code, event.code);
            assert_eq!(shortcut.modifiers, event.modifiers);
        }
    }
}
