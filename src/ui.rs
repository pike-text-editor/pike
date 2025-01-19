use ratatui::{
    buffer::Buffer,
    layout::{Position as TerminalPosition, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text, ToText},
    widgets::{self, Paragraph, StatefulWidget, Widget},
};
use scribe::buffer::Position as BufferPosition;
use std::rc::Rc;
use std::{cmp::min, path::PathBuf};
use tui_input::{Input, InputRequest};

use crate::pike::Highlight;

/// We would like to have some struct which can be rendered
/// as a list with given callbacks to be executed when something is
/// selected (similarly to telescope.nvim) so that we can reuse
/// it when searching for files or word occurrences in the cwd
/// As of now, I can't look that far into the future without writing
/// some code to know what fields this should keep and how it should
/// behave, so it's empty
#[allow(dead_code)]
struct Picker {}

const HIGHLIGHT_BG_SELECTED: Color = Color::Rgb(245, 206, 88);
const HIGHLIGHT_BG_UNSELECTED: Color = Color::Rgb(240, 137, 48);

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
    pub search_input: Option<Input>,
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

    pub fn update_highlights(&mut self, highlights: Vec<Highlight>) {
        self.buffer_state.highlight_state.highlights = highlights;
        let focused_highlight = self.buffer_state.highlight_state.focused;
        self.buffer_state.highlight_state.highlights[focused_highlight].is_selected = true;
    }

    pub fn focused_highlight_position(&mut self) -> BufferPosition {
        let focused_highlight = self.buffer_state.highlight_state.focused;
        let highlight = &self.buffer_state.highlight_state.highlights[focused_highlight];
        highlight.start
    }

    pub fn focus_next_highlight(&mut self) {
        let highlights = &mut self.buffer_state.highlight_state.highlights;
        let currently_focused = self.buffer_state.highlight_state.focused;
        let n_of_highlights = highlights.len();

        highlights[currently_focused].is_selected = false;

        let next_highlight = currently_focused.wrapping_add(1) % n_of_highlights;
        highlights[next_highlight].is_selected = true;
        self.buffer_state.highlight_state.focused = next_highlight;
    }

    pub fn focus_prev_highlight(&mut self) {
        let highlights = &mut self.buffer_state.highlight_state.highlights;
        let currently_focused = self.buffer_state.highlight_state.focused;
        let n_of_highlights = highlights.len();

        highlights[currently_focused].is_selected = false;

        let prev_highlight = currently_focused.wrapping_sub(1) % n_of_highlights;
        highlights[prev_highlight].is_selected = true;
        self.buffer_state.highlight_state.focused = prev_highlight;
    }

    pub fn clear_highlights(&mut self) {
        self.buffer_state.highlight_state.highlights.clear();
        self.buffer_state.highlight_state.focused = 0;
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
pub struct HighlightState {
    pub highlights: Vec<Highlight>,
    pub focused: usize,
}

#[derive(Default)]
/// Represents the state of a buffer display, including its contents, cursor position, and offset.
pub struct BufferDisplayState {
    pub offset: BufferDisplayOffset,
    pub highlight_state: HighlightState,
}

#[allow(dead_code)]
impl BufferDisplayState {
    pub fn new(offset: BufferDisplayOffset) -> Self {
        BufferDisplayState {
            offset,
            highlight_state: HighlightState::default(),
        }
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
    pub fn add_highlights<'a>(&self, contents: &'a str, highlights: &[Highlight]) -> Text<'a> {
        let mut highlighted_content = vec![];
        let contents_to_lines = contents.lines().collect::<Vec<&str>>();

        for (line_index, line_text) in contents_to_lines.iter().enumerate() {
            let mut line = Vec::new();
            let mut current_pos = 0;

            // Find all highlights in this line, considering the offset
            for highlight in highlights
                .iter()
                .filter(|h| h.start.line == line_index + self.offset.y)
            {
                let highlight_start = highlight.start.offset.saturating_sub(self.offset.x);
                let highlight_end = (highlight_start + highlight.length).min(line_text.len());

                if highlight_start > current_pos {
                    // Add unhighlighted text before the highlight
                    line.push(Span::raw(&line_text[current_pos..highlight_start]));
                }

                let highlight_bg = if highlight.is_selected {
                    HIGHLIGHT_BG_SELECTED
                } else {
                    HIGHLIGHT_BG_UNSELECTED
                };

                // Add highlighted text
                line.push(Span::styled(
                    &line_text[highlight_start..highlight_end],
                    Style::default()
                        .fg(Color::Black)
                        .bg(highlight_bg)
                        .add_modifier(Modifier::BOLD),
                ));

                current_pos = highlight_end;
            }

            // Add the remaining unhighlighted text
            if current_pos < line_text.len() {
                line.push(Span::raw(&line_text[current_pos..]));
            }

            highlighted_content.push(Line::from(line));
        }

        Text::from(highlighted_content)
    }

    fn prepare_paragraph_widget<'a>(&mut self, contents: &'a str) -> Paragraph<'a> {
        let paragraph_widget = if !self.highlight_state.highlights.is_empty() {
            let text_widget = self.add_highlights(contents, &self.highlight_state.highlights);
            Paragraph::new(text_widget)
        } else {
            let text_widget = Text::from(contents);
            Paragraph::new(text_widget)
        };
        paragraph_widget
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

        let paragraph_widget = state.prepare_paragraph_widget(&shifted_contents);
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

#[derive(Default)]
pub struct SearchInput {}

impl StatefulWidget for SearchInput {
    type State = Input;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let widget = widgets::Paragraph::new(state.to_text()).block(
            widgets::Block::new()
                .borders(widgets::Borders::all())
                .title("Search for: "),
        );
        widget.render(area, buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_util::ui::{n_spaces, nth_line_from_terminal_buffer, vertical_border},
        ui::{BufferDisplayState, FileInputRole, FileInputState},
    };
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};
    use scribe::buffer::Position as BufferPosition;
    use tui_input::InputRequest;

    use crate::pike::Highlight;

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

    #[test]
    fn test_add_highlights_single_line_unselected() {
        // Setup a default display state with no offset.
        let state = BufferDisplayState::default();

        let content = "Hello world";
        let highlight = Highlight {
            start: BufferPosition { line: 0, offset: 6 },
            length: 5,
            is_selected: false,
        };

        let text = state.add_highlights(content, &[highlight]);

        // Extract spans and styles from the resulting Text.
        let lines: Vec<_> = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| (span.content.clone(), span.style))
                    .collect::<Vec<_>>()
            })
            .collect();

        // We expect one line with two spans: unhighlighted "Hello " and highlighted "world".
        assert_eq!(lines.len(), 1);
        let spans = &lines[0];
        assert_eq!(spans.len(), 2);

        // First span: "Hello " with default style.
        assert_eq!(spans[0].0, "Hello ");
        assert_eq!(spans[0].1, Style::default());

        // Second span: "world" with highlighted style.
        let expected_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(240, 137, 48))
            .add_modifier(Modifier::BOLD);
        assert_eq!(spans[1].0, "world");
        assert_eq!(spans[1].1, expected_style);
    }

    #[test]
    fn test_add_highlights_single_line_selected() {
        // Test with a selected highlight which uses a different background color.
        let state = BufferDisplayState::default();

        let content = "Hello world";
        let highlight = Highlight {
            start: BufferPosition { line: 0, offset: 6 },
            length: 5,
            is_selected: true,
        };

        let text = state.add_highlights(content, &[highlight]);

        let lines: Vec<_> = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| (span.content.clone(), span.style))
                    .collect::<Vec<_>>()
            })
            .collect();

        assert_eq!(lines.len(), 1);
        let spans = &lines[0];
        assert_eq!(spans.len(), 2);

        // First span: "Hello " with default style.
        assert_eq!(spans[0].0, "Hello ");
        assert_eq!(spans[0].1, Style::default());

        // Second span: "world" with selected highlight style.
        let expected_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(245, 206, 88)) // Selected color
            .add_modifier(Modifier::BOLD);
        assert_eq!(spans[1].0, "world");
        assert_eq!(spans[1].1, expected_style);
    }

    #[test]
    fn test_add_highlights_multiple_lines_and_highlights() {
        let state = BufferDisplayState::default();

        let content = "Line one\nLine two\nLine three";
        let highlights = vec![
            Highlight {
                start: BufferPosition { line: 0, offset: 5 },
                length: 3,
                is_selected: false,
            },
            Highlight {
                start: BufferPosition { line: 1, offset: 5 },
                length: 3,
                is_selected: true,
            },
        ];

        let text = state.add_highlights(content, &highlights);

        let lines: Vec<_> = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| (span.content.clone(), span.style))
                    .collect::<Vec<_>>()
            })
            .collect();

        // Check first line spans.
        // "Line one" with a highlight on "one"
        assert!(lines.len() >= 2);
        let first_line = &lines[0];
        // Expected spans for first line: ["Line ", highlighted("one")]
        assert_eq!(first_line.len(), 2);
        assert_eq!(first_line[0].0, "Line ");
        assert_eq!(first_line[0].1, Style::default());
        let expected_style_first = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(240, 137, 48))
            .add_modifier(Modifier::BOLD);
        assert_eq!(first_line[1].0, "one");
        assert_eq!(first_line[1].1, expected_style_first);

        // Check second line spans.
        // "Line two" with a selected highlight on "two"
        let second_line = &lines[1];
        assert_eq!(second_line.len(), 2);
        assert_eq!(second_line[0].0, "Line ");
        assert_eq!(second_line[0].1, Style::default());
        let expected_style_second = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(245, 206, 88)) // Selected highlight color
            .add_modifier(Modifier::BOLD);
        assert_eq!(second_line[1].0, "two");
        assert_eq!(second_line[1].1, expected_style_second);
    }
}
