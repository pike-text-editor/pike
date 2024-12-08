use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Represents a single shortcut consisting of a key and modifiers
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct KeyShortcut {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[allow(dead_code)]
impl KeyShortcut {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyShortcut {
        KeyShortcut { code, modifiers }
    }

    /// Creates a new KeyShortcut from a crossterm::event::KeyEvent
    pub fn from_key_event(event: &KeyEvent) -> KeyShortcut {
        KeyShortcut::new(event.code, event.modifiers)
    }

    /// Creates a new KeyShortcut based on a string from a config file.
    /// String representation of the shortcut follows the VSCode notation,
    /// e.g. ctrl+shift+p, ctrl+alt+del
    pub fn from_string(s: &str) -> Result<KeyShortcut, String> {
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
