## Running the app

Running pike with a filename given as an argument opens it in the app.
The current working directory is set to the directory from which the app was
ran.

Running pike with a directory name as an argument opens the app with the current
working directory set as the argument with no file open.

Running pike with no arguments launches it with no file open in the directory the
app was launched from.

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

| Action                    | Description                                                       | Default                  | Config definition                |
|---------------------------|-------------------------------------------------------------------|--------------------------|----------------------------------|
| Open file                 | Opens a popup where a relative path should be inserted           | ctrl+o                   | "open_file"                     |
| Open new buffer           | Creates a new, empty buffer not bound to a file for editing      | ctrl+n                   | "new_buffer"                    |
| Switch to next buffer     | Moves focus to the next buffer in the list                       | ctrl+h                   | "next_buffer"                   |
| Switch to previous buffer | Moves focus to the previous buffer in the list                   | ctrl+l                   | "previous_buffer"               |
| Search in current buffer  | Searches for a specific term within the currently active buffer  | ctrl+f                   | "search_in_current_buffer"      |
| Save changes              | Saves the current buffer to its associated file                 | ctrl+s                   | "save"                          |
| Undo last change          | Reverts the most recent change in the current buffer            | ctrl+z                   | "undo"                          |
| Redo last change          | Reapplies the most recently undone change in the current buffer | ctrl+y                   | "redo"                          |
| Quit                      | Closes the application                                           | ctrl+q                   | "quit"                          |

Keybindings which contain multiple modifiers are not yet supported and will be added
in the future (<https://docs.rs/crossterm/latest/crossterm/event/struct.KeyboardEnhancementFlags.html>).

## Search utility

To search in the current buffer, press your corresponding keybind and enter the search term.
Press enter and toggle between highlighted results by pressing right/left arrow keys. Press escape to quit searching. The cursor
is moved to the currently highlighted search term.

## Unsaved changes

The app displays an indicator `*` of whether the buffer has been modified since being read/written from the filesystem in
the status bar. Quitting with unsaved changes **will discard them**, this is to be modified in the future.
