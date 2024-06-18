use std::fs::read_to_string;
use std::io::Error;

use super::line::Line;
use super::Location;


#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load_file(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self { lines })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        if at.line_index > self.height() {
            return;
        }

        if at.line_index == self.height() {
            self.lines.push(Line::from(&character.to_string()));
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
        }
    }

    pub fn delete_char(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count() && self.height() > at.line_index.saturating_add(1) {
                // to check if we are at the end of the line
                let next_line = self.lines.remove(at.line_index.saturating_add(1));

                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].append(&next_line);
            } else if at.grapheme_index < line.grapheme_count() {
                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].delete_char(at.grapheme_index);
            }
        }
    }

    pub fn insert_new_line(&mut self, at: Location) {
        if at.line_index == self.height() {
            // if we are at the end of the document,
            // which means we are at the last line,
            // insert a new empty line
            self.lines.push(Line::default());
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let new = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new);

        }
    }
}
