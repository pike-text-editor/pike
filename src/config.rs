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
        todo!()
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
