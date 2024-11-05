#[allow(dead_code, unused_variables, unused_mut)]
/// Every keymappable operation within pike
#[derive(Debug)]
pub enum Operation {
    CreateNewFile,

    CreateNewBuffer,
    SwitchToPreviousBuffer,
    SwitchToNextBuffer,
    OpenBufferPicker,

    SearchInCurrentBuffer,
    SearchAndReplaceInCurrentBuffer,

    SaveBufferToFile,

    Undo,
    Redo,

    FindFilesInCWD,
    FindTextInCWD,
}

#[allow(dead_code, unused_variables, unused_mut)]
impl Operation {
    /// Creates a new Operation from a string from a config file
    pub fn from_string(query: &str) -> Option<Operation> {
        match query {
            "new_file" => Some(Operation::CreateNewFile),
            "new_buffer" => Some(Operation::CreateNewBuffer),
            "previous_buffer" => Some(Operation::SwitchToPreviousBuffer),
            "next_buffer" => Some(Operation::SwitchToNextBuffer),
            "open_buffer_picker" => Some(Operation::OpenBufferPicker),
            "search_in_current_buffer" => Some(Operation::SearchInCurrentBuffer),
            "replace_in_current_buffer" => Some(Operation::SearchAndReplaceInCurrentBuffer),
            "save" => Some(Operation::SaveBufferToFile),
            "undo" => Some(Operation::Undo),
            "redo" => Some(Operation::Redo),
            "find_files_in_cwd" => Some(Operation::FindFilesInCWD),
            "find_text_in_cwd" => Some(Operation::FindTextInCWD),
            _ => None,
        }
    }
}
