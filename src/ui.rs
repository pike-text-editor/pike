use ratatui::{
    layout::Position as TerminalPosition,
    text::Text,
    widgets::{Paragraph, StatefulWidget, Widget},
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
    x: usize,
    /// Y offset of the entire buffer
    y: usize,
}

#[allow(dead_code)]
impl BufferDisplayOffset {
    pub fn new(x: usize, y: usize) -> Self {
        BufferDisplayOffset { x, y }
    }
}

#[derive(Default)]
pub struct BufferDisplayState {
    pub buffer_contents: String,
    pub cursor_position: Option<BufferPosition>,
    pub offset: BufferDisplayOffset,
}

impl BufferDisplayState {
    pub fn new(
        buffer_contents: String,
        cursor_position: Option<BufferPosition>,
        offset: BufferDisplayOffset,
    ) -> Self {
        BufferDisplayState {
            buffer_contents,
            cursor_position,
            offset,
        }
    }
    /// Updates the x offset of the buffer so that the cursor is always visible
    fn update_x_offset(&mut self, area: ratatui::prelude::Rect) {
        let cursor_x = self.cursor_position.as_ref().map_or(0, |pos| pos.offset);

        let too_far_right = cursor_x as u16 >= self.offset.x as u16 + area.width;
        if too_far_right {
            self.offset.x = cursor_x
                .saturating_sub(area.width as usize)
                .saturating_add(1);
        }

        // Ensure offset.x is never greater than cursor_x
        self.offset.x = min(self.offset.x, cursor_x);
    }

    /// Updates the y offset of the buffer so that the cursor is always visible
    fn update_y_offset(&mut self, area: ratatui::prelude::Rect) {
        let cursor_y = self.cursor_position.as_ref().map_or(0, |pos| pos.line);

        let too_far_down = cursor_y as u16 >= self.offset.y as u16 + area.height;
        if too_far_down {
            self.offset.y = cursor_y
                .saturating_sub(area.height as usize)
                .saturating_add(1);
        }

        // Ensure offset.y is never greater than cursor_y
        self.offset.y = min(self.offset.y, cursor_y);
    }

    /// Shifts the content of the buffer down by the offset and returns the resulting string.
    /// Basically removes the first self.offset.y lines and joins the remanining ones.
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

    /// Calculates the render position of the cursor in the given rect.
    pub fn calculate_cursor_render_position(
        &self,
        area: ratatui::layout::Rect,
    ) -> TerminalPosition {
        use std::cmp::min;

        let (max_x, max_y) = (area.width.saturating_sub(1), area.height.saturating_sub(1));
        let (base_x, base_y) = (area.x, area.y);

        let cursor_position = self
            .cursor_position
            .as_ref()
            .unwrap_or(&BufferPosition { line: 0, offset: 0 });

        TerminalPosition {
            x: min(
                (base_x + cursor_position.offset as u16).saturating_sub(self.offset.x as u16),
                max_x,
            ),
            y: min(
                (base_y + cursor_position.line as u16).saturating_sub(self.offset.y as u16),
                max_y,
            ),
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
}

/// Widget for displaying the buffer contents. Serves as a thin wrapper
/// to lift the responsibility of actually rendering the contents from the
/// app itself, so it does not copy the data, but itself receives references
/// as it's created on every app render.

pub struct BufferDisplayWidget;

impl StatefulWidget for BufferDisplayWidget {
    /// For this example, we won't store extra "widget state"
    /// outside of what's already in `BufferDisplay`, so we use `()`.
    type State = BufferDisplayState;

    /// Render the widget. Notice that `render` here includes a `state` parameter
    /// which you can use if you need separate "runtime" state.
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // Update offsets to keep cursor visible
        state.update_x_offset(area);
        state.update_y_offset(area);

        // Shift contents based on offset
        let shifted_contents = state.shift_contents(state.buffer_contents.clone());
        // Render the text using Paragraph
        let text_widget = Text::from(shifted_contents);
        let paragraph_widget = Paragraph::new(text_widget);

        // We need the Widget trait to be in scope for .render(...) to work
        paragraph_widget.render(area, buf);
    }
}
