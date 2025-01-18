#[allow(dead_code, unused_variables, unused_mut)]
/// Every keymappable operation within pike
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Operation {
    OpenFile,
    CreateNewBuffer,
    SwitchToPreviousBuffer,
    SwitchToNextBuffer,
    SearchInCurrentBuffer,
    SaveBufferToFile,
    Undo,
    Redo,
    Quit,
}

#[allow(dead_code, unused_variables, unused_mut)]
impl Operation {
    /// Creates a new Operation from a string from a config file
    pub fn from_string(query: &str) -> Result<Operation, String> {
        let return_value = match query {
            "open_file" => Operation::OpenFile,
            "new_buffer" => Operation::CreateNewBuffer,
            "previous_buffer" => Operation::SwitchToPreviousBuffer,
            "next_buffer" => Operation::SwitchToNextBuffer,
            "search_in_current_buffer" => Operation::SearchInCurrentBuffer,
            "save" => Operation::SaveBufferToFile,
            "undo" => Operation::Undo,
            "redo" => Operation::Redo,
            "quit" => Operation::Quit,
            _ => return Err(format!("Invalid operation in config: {query}")),
        };
        Ok(return_value)
    }
}
