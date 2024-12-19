use ratatui::{
    text::Text,
    widgets::{Paragraph, Widget},
};
use scribe::buffer::Position;

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

/// Holds the information about the current state of the UI
/// of the app.
#[allow(dead_code)]
#[derive(Default)]
pub struct UIState {
    /// Offset of the currently rendered buffer
    pub buffer_offset: BufferDisplayOffset,
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
    cursor_position: Option<&'a Position>,
    /// Offset of the buffer being rendered
    offset: &'a BufferDisplayOffset,
}

impl BufferDisplay<'_> {
    pub fn new<'a>(
        buffer_contents: &'a str,
        cursor_position: Option<&'a Position>,
        offset: &'a BufferDisplayOffset,
    ) -> BufferDisplay<'a> {
        BufferDisplay {
            buffer_contents,
            cursor_position,
            offset,
        }
    }
}

impl Widget for BufferDisplay<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let text_widget = Text::from(self.buffer_contents);
        let paragraph_widget = Paragraph::new(text_widget);
        paragraph_widget.render(area, buf);
    }
}
