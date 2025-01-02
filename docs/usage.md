## Configuration

The editor is configured using a toml file, by default searched for at `$XDG_CONFIG_HOME/pike/pike.toml`.
You can run pike with `-c/--config [path]` for a custom config path.

### Keymap

Located in the `keymaps` section. The keybinds follow VSCode notation, e.g. `ctrl+shift+p`, `ctrl+alt+del` and
are defined as key-value pairs. Example:

```toml
[keymaps]
"ctrl+s" = "open_file"
"ctrl+y" = "open_file"
```

The following actions are bindable:

| Action | Description | Default | Config definition |
| --------------- | --------------- | --------------- | --------------- |
| Open file | Item2.1 | Item3.1 | Item4.1 |
| Open new buffer | Item2.2 | Item3.2 | Item4.2 |
| Switch to next buffer | Item2.3 | Item3.3 | Item4.3 |
| Switch to previous buffer | Item2.4 | Item3.4 | Item4.4 |
| Search in current buffer | Item2.4 | Item3.4 | Item4.4 |
| Replace in current buffer | Item2.4 | Item3.4 | Item4.4 |
| Save changes | Item2.4 | Item3.4 | Item4.4 |
| Undo last change | Item2.4 | Item3.4 | Item4.4 |
| Redo last change | Item2.4 | Item3.4 | Item4.4 |
| Quit | Item2.4 | Item3.4 | Item4.4 |

* "open_file" opens a text input to enter the relative path of the file
to be opened. Default:
* "new_buffer" creates an empty buffer

Note that if you want to create a keybinding which contains multiple modifiers (ctrl+alt, alt+shift, etc),
you need to have a <!-- TODO: https://docs.rs/crossterm/latest/crossterm/event/struct.KeyboardEnhancementFlags.html -->
