# Pike - Perfectly Incomplex Konsole Editor

Pike is a simple, uncomplicated, and easily configurable command-line text editor designed to be a lightweight and user-friendly replacement for nano. It supports basic text editing features with intuitive keyboard shortcuts and a clear terminal interface.

![Linux](https://github.com/pike-text-editor/pike/actions/workflows/linux-ci.yaml/badge.svg)  
![Windows](https://github.com/pike-text-editor/pike/actions/workflows/windows-ci.yaml/badge.svg)

## Features

- Simple and intuitive terminal interface.
- Highlighting and search functionality within buffers.
- Undo/redo support for efficient editing.
- Configurable keyboard shortcuts.
- Cross-platform support for Linux and Windows.

---

## Installation

### Building from Source

To build and install Pike, you need Rust installed on your system. Follow the instructions for your platform:

### Linux

1. Clone the repository:
   ```bash
   git clone https://github.com/pike-text-editor/pike.git
   cd pike
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary (requires sudo for system-wide installation):
   ```bash
   sudo cp target/release/pike /usr/local/bin
   ```

4. Run Pike:
   ```bash
   pike
   ```

### Windows

1. Clone the repository:
   ```powershell
   git clone https://github.com/pike-text-editor/pike.git
   cd pike
   ```

2. Build the project using [cross](https://github.com/cross-rs/cross) or Rust's native Windows toolchain:
   ```powershell
   cargo build --release
   ```

3. Copy the binary to a location in your PATH:
   ```powershell
   copy .\target\release\pike.exe C:\Path\To\Bin
   ```

4. Run Pike:
   ```powershell
   pike
   ```

---

## Documentation

Comprehensive documentation, including usage instructions and examples, is available in the [`docs/usage.md`](docs/usage.md) file. It covers:

- How to use Pike effectively.
- Configuration options for customizing shortcuts and behavior.

---

## Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests to enhance Pike.

---

Start using Pike as your go-to command-line text editor today!