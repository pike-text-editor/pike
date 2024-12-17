use std::io::Write;

use tempfile::NamedTempFile;

#[allow(dead_code)] // TODO: why does clippy not see it being used in private modules?
/// Create a temporary file with the given contents
pub fn temp_file_with_contents(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(contents.as_bytes())
        .expect("Failed to write to temp file");
    file
}
