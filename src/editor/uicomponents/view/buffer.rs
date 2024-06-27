use super::super::super::AnnotatedString;
use super::{FileInfo, Highlighter, Line};
use crate::prelude::*;
use std::{
    fs::{read_to_string, File},
    io::{Error, Write},
    ops::Range,
};
#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,    // vector of lines in the buffer, including the whole document
    file_info: FileInfo, // file info of the document in the current buffer
    dirty: bool, // to indicate whether the buffer is modified or not, default is false, set to true when buffer is modified
}

impl Buffer {
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub const fn get_file_info(&self) -> &FileInfo {
        &self.file_info
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn grapheme_count(&self, idx: LineIdx) -> GraphemeIdx {
        self.lines.get(idx).map_or(0, Line::grapheme_count)
    }

    pub fn width_until(&self, idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(idx)
            .map_or(0, |line| line.width_until(until))
    }

    pub fn get_highlighted_substring(
        &self,
        line_idx: LineIdx,
        range: Range<GraphemeIdx>,
        // search_results: &Option<Vec<GraphemeIdx>>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_idx).map(|line| {
            line.get_annotated_visible_substr(
                range,
                Some(&highlighter.get_annotations(line_idx)),
                // search_results,
            )
        })
    }

    pub fn highlight(&self, idx: LineIdx, search_results: &Option<Vec<GraphemeIdx>>, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(idx) {
            highlighter.highlight(idx, line, search_results);
        }
    }

    pub fn load_file(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();

        for value in contents.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self {
            lines,
            file_info: FileInfo::from(file_name),
            dirty: false,
        })
    }

    pub fn search(&mut self, query: &str) -> Option<Vec<Location>> {
        let mut locations = Vec::new();

        for (line_index, line) in self.lines.iter().enumerate() {
            if let Some(grapheme_indices) = line.search(query) {
                for grapheme_index in grapheme_indices {
                    locations.push(Location {
                        line_index,
                        grapheme_index,
                    });
                }
            }
        }

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    fn save_to_file(&self, file_info: &FileInfo) -> Result<(), Error> {
        if let Some(file_path) = &file_info.get_path() {
            let mut file = File::create(file_path)?;

            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }

        Ok(())
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
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
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
            self.dirty = true;
        }
    }

    pub fn delete_char(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count()
                && self.height() > at.line_index.saturating_add(1)
            {
                // to check if we are at the end of the line
                let next_line = self.lines.remove(at.line_index.saturating_add(1));

                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_index < line.grapheme_count() {
                #[allow(clippy::integer_arithmetic)]
                self.lines[at.line_index].delete_char(at.grapheme_index);
                self.dirty = true;
            }
        }
    }

    pub fn insert_new_line(&mut self, at: Location) {
        if at.line_index == self.height() {
            // if we are at the end of the document,
            // which means we are at the last line,
            // insert a new empty line
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let new = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new);
            self.dirty = true;
        }
    }
}
