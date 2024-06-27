use super::super::{
    command::{Edit, Move},
    DocumentStatus, Line, Terminal,
};
use super::UIComponent;
use crate::editor::RowIdx;
use crate::prelude::*;
use buffer::Buffer;
use fileinfo::FileInfo;
use searchinfo::SearchInfo;
use std::{cmp::min, io::Error};
use highlighter::Highlighter;

mod buffer;
mod fileinfo;
mod searchinfo;
mod highlighter;

// #[derive(Clone, Copy, Default, Debug)]
// pub struct Location {
//     pub grapheme_index: usize,
//     pub line_index: usize,
// }

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    prev_text_location: Location,
    text_location: Location,
    scroll_offset: Position,
    search_info: Option<SearchInfo>,
    size: Size,
}

impl View {
    pub fn get_status(&self) -> DocumentStatus {
        let file_info = self.buffer.get_file_info();

        DocumentStatus {
            current_grapheme_index: self.text_location.grapheme_index,
            current_line_index: self.text_location.line_index,
            file_name: format!("{file_info}"),
            file_type: file_info.get_file_type(),
            is_modified: self.buffer.is_dirty(),
            total_lines: self.buffer.height(),
        }
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    // SECTION: search

    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            current_idx: None,
            prev_location: self.text_location,
            prev_scroll_offset: self.scroll_offset,
            query: None,
            result: None,
        });
    }

    pub fn exit_search(&mut self) {
        self.search_info = None;
        self.set_needs_redraw(true);
    }

    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.prev_location;
        }

        self.exit_search();
        // self.search_info = None;
        // self.set_needs_redraw(true);
        // self.scroll_text_location_into_view();
    }

    pub fn search(&mut self, query: &str) {
        if query.is_empty() {
            self.search_info = None;
            self.set_needs_redraw(true);
            return;
        }

        if let Some(location) = self.buffer.search(query) {
            self.search_info = Some(SearchInfo {
                current_idx: Some(0),
                prev_location: self.text_location,
                prev_scroll_offset: self.scroll_offset,
                query: Some(Line::from(query)),
                result: Some(location),
            });
        } else {
            self.search_info = None;
        }

        if let Some(search_info) = &self.search_info {
            if let Some(location) = &search_info.result {
                self.text_location = location[search_info.current_idx.unwrap()];
            }
        }
        self.set_needs_redraw(true);
        self.scroll_text_location_into_view();
    }

    pub fn next_search_result(&mut self) {
        if let Some(search_info) = &mut self.search_info {
            if let Some(location) = &search_info.result {
                let len: usize = location.len();
                search_info.current_idx = Some((search_info.current_idx.unwrap() + 1) % len);
                self.text_location = location[search_info.current_idx.unwrap()];
                self.set_needs_redraw(true);
            }
        }
    }

    pub fn prev_search_result(&mut self) {
        if let Some(search_info) = &mut self.search_info {
            if let Some(location) = &search_info.result {
                let len: usize = location.len();
                search_info.current_idx = Some((search_info.current_idx.unwrap() + len - 1) % len);
                self.text_location = location[search_info.current_idx.unwrap()];
                self.set_needs_redraw(true);
            }
        }
    }

    // END SECTION

    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.insert_char(character),
            Edit::DeleteBackward => self.delete_char_backward(),
            Edit::Delete => self.delete_char(),
            Edit::InsertNewline => self.insert_new_line(),
        }
    }

    pub fn handle_move_command(&mut self, command: Move) {
        let Size { height, .. } = self.size;
        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
        }

        self.scroll_text_location_into_view();
    }

    fn delete_char_backward(&mut self) {
        if self.text_location.line_index != 0 || self.text_location.grapheme_index != 0 {
            self.handle_move_command(Move::Left);
            self.delete_char();
        }
    }

    fn delete_char(&mut self) {
        self.buffer.delete_char(self.text_location);
        self.set_needs_redraw(true);
    }

    fn insert_char(&mut self, character: char) {
        let old_len = self.buffer.grapheme_count(self.text_location.line_index);
        // let old_len = self
        //     .buffer
        //     .lines
        //     .get(self.text_location.line_index)
        //     .map_or(0, Line::grapheme_count);

        self.buffer.insert_char(character, self.text_location);

        let new_len = self.buffer.grapheme_count(self.text_location.line_index);

        // let new_len = self
        //     .buffer
        //     .lines
        //     .get(self.text_location.line_index)
        //     .map_or(0, Line::grapheme_count);

        let grapheme_delta = new_len.saturating_sub(old_len);

        if grapheme_delta > 0 {
            self.handle_move_command(Move::Right);
        }

        self.set_needs_redraw(true);
    }

    fn insert_new_line(&mut self) {
        self.buffer.insert_new_line(self.text_location);
        self.handle_move_command(Move::Right);
        self.set_needs_redraw(true);
    }

    fn render_line(at: RowIdx, lines: &str) -> Result<(), Error> {
        Terminal::print_row(at, lines)
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let welcome_message: String = format!("{NAME} - version {VERSION}");
        let len: usize = welcome_message.len();

        let remaining_width: usize = width.saturating_sub(1);

        if remaining_width <= len {
            return "~".to_string();
        }

        format!("{:<1}{:^remaining_width$}", "~", welcome_message)
    }

    fn scroll_vertically(&mut self, to: RowIdx) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };

        if offset_changed {
            self.set_needs_redraw(true);
        }

        self.needs_redraw = offset_changed || self.needs_redraw;
    }

    fn scroll_horizontally(&mut self, to: ColIdx) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };

        if offset_changed {
            self.set_needs_redraw(true);
        }
        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();

        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.width_until(row, self.text_location.grapheme_index);
        Position { row, col }
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.text_location.grapheme_index = min(
            self.buffer.grapheme_count(self.text_location.line_index),
            self.prev_text_location.grapheme_index,
        );
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.text_location.grapheme_index = min(
            self.buffer
                .grapheme_count(min(self.text_location.line_index, self.buffer.len() - 1)),
            self.prev_text_location.grapheme_index,
        );
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let grapheme_count = self
            .buffer
            .grapheme_count(self.text_location.line_index);

        if self.text_location.grapheme_index < grapheme_count {
            self.text_location.grapheme_index += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
        self.prev_text_location = self.text_location;
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }

        self.prev_text_location = self.text_location;
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .grapheme_count(self.text_location.line_index);
    }

    // Ensures self.location.grapheme_index points to a valid grapheme index by snapping it to the left most grapheme if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = min(self.text_location.grapheme_index, self.buffer.grapheme_count(self.text_location.line_index));
    }

    // Ensures self.location.line_index points to a valid line index by snapping it to the bottom most line if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

    pub fn load_file(&mut self, file_name: &str) -> Result<(), Error> {
        let buffer = Buffer::load_file(file_name)?;
        self.buffer = buffer;
        self.set_needs_redraw(true);
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)
    }
}

impl UIComponent for View {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        let Size { width, height } = self.size;
        // assert_eq!(self.scroll_offset.row, 0);
        let end_y = origin_row.saturating_add(height);

        #[allow(clippy::integer_division)]
        let top_third = height / 3;
        let scroll_top = self.scroll_offset.row;

        let query: Option<&str> = self
                    .search_info
                    .as_ref()
                    .and_then(|search_info| search_info.query.as_deref());

        let selected_match = query.is_some().then_some(self.text_location);

        
        let mut highlighter = Highlighter::new(query, selected_match);
        
        for current_row in 0..end_y {
            let search_results = if let Some(search_info) = &self.search_info {
                    if let Some(locations) = &search_info.result {
                        let res = locations
                            .iter()
                            .filter(|location| location.line_index == current_row)
                            .map(|location| location.grapheme_index)
                            .collect::<Vec<_>>();
                        Some(res)
                    } else {
                        None
                    }
                } else {
                    None
                };

            self.buffer.highlight(current_row, &search_results, &mut highlighter);
        }

        for current_row in origin_row..end_y {
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);

            let left = self.scroll_offset.col;
            let right = self.scroll_offset.col.saturating_add(width);

            // self.buffer.highlight(idx, &mut highlighter)



            // if let Some(line) = self.buffer.lines.get(line_idx) {
            //     let left = self.scroll_offset.col;
            //     let right = self.scroll_offset.col.saturating_add(width);

            //     let query: Option<&str> = self
            //         .search_info
            //         .as_ref()
            //         .and_then(|search_info| search_info.query.as_deref());

            //     let selected_match = (self.text_location.line_index == line_idx && query.is_some())
            //         .then_some(self.text_location.grapheme_index);
            //     let search_results = if let Some(search_info) = &self.search_info {
            //         if let Some(locations) = &search_info.result {
            //             let res = locations
            //                 .iter()
            //                 .filter(|location| location.line_index == line_idx)
            //                 .map(|location| location.grapheme_index)
            //                 .collect::<Vec<_>>();
            //             Some(res)
            //         } else {
            //             None
            //         }
            //     } else {
            //         None
            //     };
            //     Terminal::print_annotated_row(
            //         current_row,
            //         &line.get_annotated_visible_substr(
            //             left..right,
            //             query,
            //             selected_match,
            //             &search_results,
            //         ),
            //     )?;
            if let Some(annotated_string) = self.buffer.get_highlighted_substring(line_idx, left..right, &highlighter) {
                Terminal::print_annotated_row(current_row, &annotated_string)?;
            } else if current_row == top_third && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }

        Ok(())
    }
}
