use ratatui::{
    buffer::Buffer,
    layout::{Position as TerminalPosition, Rect},
    text::{Text, ToText},
    widgets::{self, Paragraph, StatefulWidget, Widget},
};
use scribe::buffer::Position as BufferPosition;
use std::rc::Rc;
use std::{cmp::min, path::PathBuf};
use tui_input::{Input, InputRequest};

/// We would like to have some struct which can be rendered
/// as a list with given callbacks to be executed when something is
/// selected (similarly to telescope.nvim) so that we can reuse
/// it when searching for files or word occurrences in the cwd
/// As of now, I can't look that far into the future without writing
/// some code to know what fields this should keep and how it should
/// behave, so it's empty
#[allow(dead_code)]
struct Picker {}

pub enum CursorCalculationMode<'a> {
    FileInput(&'a Input),
    Buffer,
}

/// Two ways a file input can serve in the app - either when opening
/// a new file by path or saving an unbound buffer
#[derive(Debug, Clone, PartialEq)]
pub enum FileInputRole {
    GetOpenPath,
    GetSavePath,
}

/// Holds an input and an indicator of its role
#[derive(Clone)]
pub struct FileInputState {
    pub input: Input,
    pub role: FileInputRole,
}

impl FileInputState {
    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(self.input.to_string())
    }

    pub fn handle(&mut self, req: InputRequest) {
        self.input.handle(req);
    }
}

impl From<(&str, FileInputRole)> for FileInputState {
    fn from((input, role): (&str, FileInputRole)) -> Self {
        FileInputState {
            input: input.into(),
            role,
        }
    }
}

/// Holds the information about the current state of the UI
/// of the app.
#[allow(dead_code)]
#[derive(Default)]
pub struct UIState {
    /// Offset of the currently rendered buffer
    pub buffer_state: BufferDisplayState,
    /// A text input used to enter the filepath when saving an unbound buffer
    /// and opening a new file
    pub file_input: Option<FileInputState>,
}

impl UIState {
    /// Calculate the cursor position for a given `CursorCalculation` mode
    pub fn calculate_cursor_position(
        &self,
        calc_mode: CursorCalculationMode,
        layout: &Rc<[Rect]>,
        cursor_pos: Option<BufferPosition>,
    ) -> TerminalPosition {
        let main_area = 1;
        let status_bar_area = 0;

        match calc_mode {
            CursorCalculationMode::FileInput(input) => {
                self.calculate_cursor_for_file_input(input, layout[main_area])
            }
            CursorCalculationMode::Buffer => {
                self.calculate_cursor_for_buffer(layout[status_bar_area], cursor_pos)
            }
        }
    }

    /// Calculate position for file input
    pub fn calculate_cursor_for_file_input(&self, input: &Input, area: Rect) -> TerminalPosition {
        let border_offset = 1;

        let max_x = {
            let (x, _) = Self::max_rect_position(&area);
            x.saturating_sub(border_offset)
        };

        let (base_x, base_y) = {
            let (x, y) = Self::base_rect_position(&area);
            (x + border_offset, y)
        };

        let offset = input.cursor() as u16;

        TerminalPosition::new(min(base_x + offset, max_x), base_y + border_offset)
    }

    /// Calculate position for buffer
    pub fn calculate_cursor_for_buffer(
        &self,
        area: Rect,
        cursor_pos: Option<BufferPosition>,
    ) -> TerminalPosition {
        // If we have a cursor position, compute accordingly;
        // otherwise return a default
        if let Some(cursor_pos) = cursor_pos {
            let (max_x, max_y) = Self::max_rect_position(&area);
            let (base_x, base_y) = Self::base_rect_position(&area);

            let x_offset = self.buffer_state.offset.x as u16;
            let y_offset = self.buffer_state.offset.y as u16;

            let x = (base_x + cursor_pos.offset as u16).saturating_sub(x_offset);
            let y = (base_y + cursor_pos.line as u16).saturating_sub(y_offset);

            TerminalPosition {
                x: min(x, max_x),
                y: min(y, max_y),
            }
        } else {
            TerminalPosition::default()
        }
    }

    /// Calculate the maximum renderable position in a given area
    fn max_rect_position(area: &Rect) -> (u16, u16) {
        (area.width.saturating_sub(1), area.height.saturating_sub(1))
    }

    /// Calculate the base (top-left) position in a given area
    fn base_rect_position(area: &Rect) -> (u16, u16) {
        (area.x, area.y)
    }
}

/// Holds the information how much offset is the
/// current buffer when displayed - for example, it's
/// displayed from line 6 until either the end of the buffer ->
/// BufferDisplayOffset{ 0, 6 }. Used to consistently shift the buffer
/// when rendering. Persisted in UIState between renders.
#[allow(dead_code)]
#[derive(Default)]
pub struct BufferDisplayOffset {
    /// X offset of the line pointed at by the cursor
    pub x: usize,
    /// Y offset of the entire buffer
    pub y: usize,
}

#[allow(dead_code)]
impl BufferDisplayOffset {
    pub fn new(x: usize, y: usize) -> Self {
        BufferDisplayOffset { x, y }
    }
}

#[derive(Default)]
/// Represents the state of a buffer display, including its contents, cursor position, and offset.
pub struct BufferDisplayState {
    pub offset: BufferDisplayOffset,
}

#[allow(dead_code)]
impl BufferDisplayState {
    pub fn new(offset: BufferDisplayOffset) -> Self {
        BufferDisplayState { offset }
    }
    /// Updates the x offset of the buffer so that the cursor is always visible
    pub fn update_x_offset(&mut self, area: Rect, cursor_offset_x: usize) {
        let too_far_right = cursor_offset_x as u16 >= self.offset.x as u16 + area.width;
        if too_far_right {
            self.offset.x = cursor_offset_x
                .saturating_sub(area.width as usize)
                .saturating_add(1);
        }

        // Ensure offset.x is never greater than cursor_x
        self.offset.x = self.offset.x.min(cursor_offset_x);
    }

    pub fn update_y_offset(&mut self, area: Rect, cursor_line: usize) {
        let too_far_down = cursor_line as u16 >= self.offset.y as u16 + area.height;
        if too_far_down {
            self.offset.y = cursor_line
                .saturating_sub(area.height as usize)
                .saturating_add(1);
        }

        // Ensure offset.y is never greater than cursor_y
        self.offset.y = self.offset.y.min(cursor_line);
    }

    /// Shifts the content of the buffer down by the offset and returns the resulting string.
    /// Basically removes the first self.offset.y lines and joins the remaining ones.
    fn shift_contents_down(&mut self, contents: String) -> String {
        contents
            .lines()
            .skip(self.offset.y)
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Shifts the content of the buffer to the right by the offset and returns the resulting
    /// string. Basically, takes every line and removes line[0:self.offset.x] from it, then
    /// joins and returns them.
    fn shift_contents_right(&mut self, contents: String) -> String {
        contents
            .lines()
            .map(|line| {
                let line = line.chars().skip(self.offset.x).collect::<String>();
                line
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn shift_contents(&mut self, contents: String) -> String {
        let down_shifted = self.shift_contents_down(contents);
        self.shift_contents_right(down_shifted)
    }
}

/// Widget for displaying the buffer contents. Serves as a thin wrapper
/// to lift the responsibility of actually rendering the contents from the
/// app itself
pub struct BufferDisplayWidget<'a> {
    pub buffer_contents: &'a str,
    pub cursor_position: Option<BufferPosition>,
}

impl<'a> BufferDisplayWidget<'a> {
    pub fn new(buffer_contents: &'a str, cursor_position: Option<BufferPosition>) -> Self {
        Self {
            buffer_contents,
            cursor_position,
        }
    }
}

impl StatefulWidget for BufferDisplayWidget<'_> {
    type State = BufferDisplayState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // Update offsets to keep cursor visible
        if let Some(pos) = self.cursor_position {
            state.update_x_offset(area, pos.offset);
            state.update_y_offset(area, pos.line);
        }
        // Shift contents based on offset
        let shifted_contents = state.shift_contents(self.buffer_contents.to_string());
        // Render the text using Paragraph
        let text_widget = Text::from(shifted_contents);
        let paragraph_widget = Paragraph::new(text_widget);

        paragraph_widget.render(area, buf);
    }
}

/// A widget for displaying a text input passed to it as a state
/// In the future might need factoring out to accommodate other UI
/// elements that need such functionality and just have a title
/// and callback passed to it as arguments
#[derive(Default)]
pub struct FileInput {}

impl StatefulWidget for FileInput {
    type State = FileInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let widget = widgets::Paragraph::new(state.input.to_text()).block(
            widgets::Block::new()
                .borders(widgets::Borders::all())
                .title("Enter relative file path"),
        );
        widget.render(area, buf)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};
    use tui_input::InputRequest;

    use crate::{
        test_util::ui::{n_spaces, nth_line_from_terminal_buffer, vertical_border},
        ui::{FileInputRole, FileInputState},
    };

    use super::FileInput;
    // TODO: could move some BufferDisplay tests here for clarity

    #[test]
    fn file_input_displays_input() {
        let mut input_state: FileInputState = ("hello", FileInputRole::GetSavePath).into();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));

        let widget = FileInput::default();
        widget.render(buf.area, &mut buf, &mut input_state);

        // Skip the borders, we're not testing the library
        let text_line = nth_line_from_terminal_buffer(&buf, 1);

        assert_eq!(
            text_line,
            vertical_border() + "hello" + &n_spaces(3) + &vertical_border()
        );

        // Insert an additional character
        let request = InputRequest::InsertChar(',');
        input_state.handle(request);

        let widget = FileInput::default();
        widget.render(buf.area, &mut buf, &mut input_state);
        let text_line = nth_line_from_terminal_buffer(&buf, 1);

        assert_eq!(
            text_line,
            vertical_border() + "hello," + &n_spaces(2) + &vertical_border()
        );
    }
}
