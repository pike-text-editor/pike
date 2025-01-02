#[cfg(test)]
pub use crate::test_util::fs::*;

#[cfg(test)]
pub mod fs {
    use std::io::Write;
    use tempfile::NamedTempFile;
    /// Create a temporary file with the given contents
    pub fn temp_file_with_contents(contents: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(contents.as_bytes())
            .expect("Failed to write to temp file");
        file
    }
}

#[cfg(test)]
pub mod ui {
    use ratatui::buffer::Buffer;

    /// Return a string representation of a solid border of a given length.
    pub fn solid_border(length: usize) -> String {
        "─".repeat(length)
    }

    /// Return a string representation of a line filled with
    /// spaces of a given length
    pub fn n_spaces(n: usize) -> String {
        String::from(" ").repeat(n)
    }

    /// Return a string representation of a vertical border of a terminal UI
    pub fn vertical_border() -> String {
        String::from("│")
    }

    /// Return a string representation of a terminal buffer line with
    /// the given index
    pub fn nth_line_from_terminal_buffer(buf: &Buffer, n: u16) -> String {
        let width = buf.area.width;
        let line = (0..width).fold(String::from(""), |acc, x| {
            acc + buf
                .cell::<(u16, u16)>((x, n))
                .expect("Iterating from 0 to buf.width should not go out of its bounds")
                .symbol()
        });
        line
    }
}
