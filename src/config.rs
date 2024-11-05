use crossterm::event::{KeyCode, KeyModifiers};

use crate::operations::Operation;
use std::collections::HashMap;

#[derive(Debug)]
struct Keymap {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug)]
pub struct Config {
    key_mappings: HashMap<Keymap, Operation>,
}
