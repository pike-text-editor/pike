# pike (Perfectly Incomplex Konsole Editor) - Code Skeleton

#### Maksym Bieńkowski, Jędrzej Grabski

![Linux](https://github.com/pike-text-editor/pike/actions/workflows/linux-ci.yaml/badge.svg)  
![Windows](https://github.com/pike-text-editor/pike/actions/workflows/windows-ci.yaml/badge.svg)

### Module Summary:

1. **`welcome_pike.rs`**:
   - Contains the welcome banner displayed when the editor is launched without a source file.

2. **`ui.rs`**:
   - Manages the rendering of the terminal interface and its state.
   - Handles cursor position calculation, file path input, and highlighting within buffers.
   - Responsible for buffer display offsets, rendering, and interactive components such as file saving/opening and the text search window.

3. **`pike.rs`**:
   - Implements the core backend logic of the editor.
   - Manages the workspace, buffers, cursor movement, and file operations (open, save, create).
   - Supports undo/redo functionality, buffer searching, and highlight management (during search operations).

4. **`operations.rs`**:
   - Defines the set of operations supported by `pike` (e.g., opening files, creating new buffers, saving, searching, etc.).
   - Parses operations from configuration strings.

5. **`main.rs`**:
   - Entry point of the application.
   - Initializes the terminal, processes command-line arguments, and runs the main application loop.

6. **`key_shortcut.rs`**:
   - Represents keyboard shortcuts and maps them to corresponding operations.
   - Provides methods for parsing shortcuts from configuration and handling user inputs.

7. **`config.rs`**:
   - Manages the editor's configuration, including loading and parsing configuration files.
   - Handles default configuration paths and validates user-defined key mappings.

8. **`app.rs`**:
   - Combines the UI and backend to provide the main application logic.
   - Manages rendering, event handling, and user-triggered operations.
   - Integrates the buffer and file input with the terminal UI.

---

## Usage

### Using `just`

- `just lint`: Checks code using `cargo clippy`.
- `just build`: Builds the project in release mode.
- `just test`: Runs tests with all features enabled.
- `just windows-build`: Builds the project for Windows using `cross`.  
  Requires [cross](https://github.com/cross-rs/cross).
- `just windows-test`: Runs tests for Windows using `cross`.  
  Requires [cross](https://github.com/cross-rs/cross).
- `just cov`: Generates an HTML test coverage report.  
  Requires [cargo-binutils](https://github.com/rust-embedded/cargo-binutils) and [grcov](https://github.com/mozilla/grcov).

### Without `just`

- `cargo run`: Runs the project.
- `cargo test`: Runs project tests.
- `cargo doc -p pike`: Generates documentation from comments.

---

### Cross-compilation for Windows

For cross-compiling to Windows, [cross](https://github.com/cross-rs/cross) is used.  
It requires Docker or Podman on the host.  
More details in the [documentation](https://github.com/cross-rs/cross#usage).

- `cross build --target=x86_64-pc-windows-gnu`: Compiles the project for Windows using the GNU toolchain.
- `cross test --target=x86_64-pc-windows-gnu`: Runs tests for Windows in a containerized environment.

---

### Tools

#### Formatter

The project uses [rustfmt](https://github.com/rust-lang/rustfmt) for formatting.

**Installation**:
```bash
rustup update
rustup component add rustfmt
```

**Usage**:
```bash
cargo fmt           # Formats files in-place
cargo fmt --check   # Checks formatting without modifying files
```

#### Linter

The project uses [clippy](https://github.com/rust-lang/rust-clippy) as the linter.

**Installation**:
```bash
rustup update
rustup component add clippy
```

**Usage**:
```bash
cargo clippy        # Runs linter without applying suggestions
cargo clippy --fix  # Automatically applies suggestions
```

Both tools should be available in the default Rust installation.