use ratatui::{
    text::{Text, ToText},
    widgets::{self, Paragraph, StatefulWidget, Widget},
};
use scribe::buffer::Position as BufferPosition;
use std::cmp::min;

/// We would like to have some struct which can be rendered
/// as a list with given callbacks to be executed when something is
/// selected (similarly to telescope.nvim) so that we can reuse
/// it when searching for files or word occurences in the cwd
/// As of now, I can't look that far into the future without writing
/// some code to know what fields this should keep and how it shoul
/// behave, so it's empty
#[allow(dead_code)]
struct Picker {}

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

/// Holds the information about the current state of the UI
/// of the app.
#[allow(dead_code)]
#[derive(Default)]
pub struct UIState {
    /// Offset of the currently rendered buffer
    pub buffer_offset: BufferDisplayOffset,

    /// Currently open file input
    pub file_input: Option<tui_input::Input>,
}

/// Widget for displaying the buffer contents. Serves as a thin wrapper
/// to lift the responsibility of actually rendering the contents from the
/// app itself, so it does not copy the data, but itself receives references
/// as it's created on every app render.
#[allow(dead_code)]
pub struct BufferDisplay<'a> {
    /// Contents of the buffer to render
    buffer_contents: &'a str,

    /// Current position of the cursor in the buffer
    cursor_position: Option<&'a BufferPosition>,

    /// Offset of the buffer being rendered
    offset: &'a mut BufferDisplayOffset,
}

// TODO: think about refactoring to a stateful widget or widgetref
impl BufferDisplay<'_> {
    pub fn new<'a>(
        buffer_contents: &'a str,
        cursor_position: Option<&'a BufferPosition>,
        offset: &'a mut BufferDisplayOffset,
    ) -> BufferDisplay<'a> {
        BufferDisplay {
            buffer_contents,
            cursor_position,
            offset,
        }
    }

    /// Updates the x offset of the buffer so that the cursor is always visible
    fn update_x_offset(&mut self, area: ratatui::prelude::Rect) {
        let cursor_x = self
            .cursor_position
            .unwrap_or(&BufferPosition { line: 0, offset: 0 })
            .offset;

        let too_far_right = cursor_x as u16 >= self.offset.x as u16 + area.width;
        if too_far_right {
            // If the current offset is greater than the length of the current line
            // we need to adjust the offset so that the cursor is visible
            self.offset.x = cursor_x - area.width as usize + 1;
        }

        // If we're going out of sight from to the left, clamp the offset
        // with the cursor's position
        self.offset.x = min(self.offset.x, cursor_x);
    }

    /// Updates the y offset of the buffer so that the cursor is always visible
    fn update_y_offset(&mut self, area: ratatui::prelude::Rect) {
        let cursor_y = self
            .cursor_position
            .unwrap_or(&BufferPosition { line: 0, offset: 0 })
            .line;

        let too_far_down = cursor_y as u16 >= self.offset.y as u16 + area.height;
        if too_far_down {
            // If the current y coordinate of the cursor is below the visible area,
            // the buffer has to be shifted down so that the cursor is visible
            self.offset.y = cursor_y - area.height as usize + 1;
        }

        // If we're going out of sight from the top, clamp the offset
        // with the cursor's position
        self.offset.y = min(self.offset.y, cursor_y);
    }

    /// Shifts the content of the buffer down by the offset and returns the resulting string.
    /// Basically removes the first self.offset.y lines and joins the remanining ones.
    fn shift_contents_down(self, contents: String) -> String {
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
}

impl Widget for BufferDisplay<'_> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.update_x_offset(area);
        self.update_y_offset(area);
        let contents_shifted_right = self.shift_contents_right(self.buffer_contents.to_string());
        let contents_shifted_down = self.shift_contents_down(contents_shifted_right);

        let text_widget = Text::from(contents_shifted_down);
        let paragraph_widget = Paragraph::new(text_widget);
        paragraph_widget.render(area, buf);
    }
}

/// A widget for displaying a text input passed to it as a state
/// In the future might need factoring out to accomodate other UI
/// elements that need such functionality and just have a title
/// and callback passed to it as arguments
#[derive(Default)]
pub struct FileInput {}

impl StatefulWidget for FileInput {
    type State = tui_input::Input;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let widget = widgets::Paragraph::new(state.to_text()).block(
            widgets::Block::new()
                .borders(widgets::Borders::all())
                .title("Enter relative file path"),
        );
        widget.render(area, buf)
    }
}
