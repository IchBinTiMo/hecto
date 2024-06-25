use super::super::{
    command::{Edit, Move},
    Line, Position, Size, Terminal,
};
use super::UIComponent;
use std::{
    cmp::{max, min},
    io::Error,
};

#[derive(Default)]
pub struct CommandBar {
    prompt: Line,
    value: Line,
    needs_redraw: bool,
    size: Size,
    caret_position: Position,
}

impl CommandBar {
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.insert_char(character, self.caret_position_col()),
            Edit::Delete => self.delete_char(self.caret_position_col()),
            Edit::DeleteBackward => self.delete_char_backward(self.caret_position_col()),
            Edit::InsertNewline => {}
        }

        self.set_needs_redraw(true);
    }

    pub fn handle_move_command(&mut self, command: Move) {
        match command {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::StartOfLine | Move::Up | Move::PageUp => self.move_to_start_of_line(),
            Move::EndOfLine | Move::Down | Move::PageDown => self.move_to_end_of_line(),
        }
    }

    pub fn caret_position_col(&self) -> usize {
        min(self.caret_position.col, self.size.width)
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = Line::from(prompt);
        self.set_caret_postion(self.prompt.grapheme_count());
        self.set_needs_redraw(true);
    }

    pub fn clear_value(&mut self) {
        self.value = Line::default();
        self.set_needs_redraw(true);
    }

    pub fn set_caret_postion(&mut self, col: usize) {
        self.caret_position.col = col;
    }

    fn move_left(&mut self) {
        self.caret_position.col = max(
            self.caret_position.col.saturating_sub(1),
            self.prompt.grapheme_count(),
        );
    }

    fn move_right(&mut self) {
        self.caret_position.col = min(
            self.caret_position.col.saturating_add(1),
            self.value.grapheme_count() + self.prompt.grapheme_count(),
        );
    }

    fn move_to_start_of_line(&mut self) {
        self.caret_position.col = self.prompt.grapheme_count();
    }

    fn move_to_end_of_line(&mut self) {
        let max_width = self
            .prompt
            .grapheme_count()
            .saturating_add(self.value.grapheme_count());

        self.caret_position.col = min(max_width, self.size.width);
    }

    fn insert_char(&mut self, character: char, col: usize) {
        self.value
            .insert_char(character, col - self.prompt.grapheme_count());
        self.caret_position.col = col + 1;
    }

    fn delete_char(&mut self, col: usize) {
        self.value.delete_char(col - self.prompt.grapheme_count());
    }

    fn delete_char_backward(&mut self, col: usize) {
        if self.caret_position.col == self.prompt.grapheme_count() {
            return;
        }

        self.value
            .delete_char(col - self.prompt.grapheme_count() - 1);
        self.caret_position.col = col - 1;
    }
}

impl UIComponent for CommandBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        let area_for_value = self.size.width.saturating_sub(self.prompt.grapheme_count());

        let value_end = self.value.width();

        let value_start = value_end.saturating_sub(area_for_value);

        let message = format!(
            "{}{}",
            self.prompt,
            self.value.get_visible_graphemes(value_start..value_end)
        );

        let to_print = if message.len() <= self.size.width {
            message
        } else {
            String::new()
        };

        Terminal::print_row(origin, &to_print)
    }
}
