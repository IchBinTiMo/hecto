use std::fs::{read_to_string, File};
use std::io::{Error, Write};

use crate::editor::flieinfo::FileInfo;

use super::line::Line;
use super::Location;


#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: bool, // to indicate whether the buffer is modified or not, default is false, set to true when buffer is modified
}

impl Buffer {
    pub fn load_file(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self { lines, file_info: FileInfo::from(file_name), dirty: false })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.file_info.path {
            let mut file = File::create(path)?;

            for line in &self.lines {
                writeln!(file, "{line}")?;
            }

            self.dirty = false;
        }

        Ok(())
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
            self.dirty = false;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
            self.dirty = false;
        }
    }

    pub fn delete_char(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count() && self.height() > at.line_index.saturating_add(1) {
                // to check if we are at the end of the line
                let next_line = self.lines.remove(at.line_index.saturating_add(1));

                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].append(&next_line);
                self.dirty = false;
            } else if at.grapheme_index < line.grapheme_count() {
                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].delete_char(at.grapheme_index);
                self.dirty = false;
            }
        }
    }

    pub fn insert_new_line(&mut self, at: Location) {
        if at.line_index == self.height() {
            // if we are at the end of the document,
            // which means we are at the last line,
            // insert a new empty line
            self.lines.push(Line::default());
            self.dirty = false;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let new = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new);
            self.dirty = false;

        }
    }
}
