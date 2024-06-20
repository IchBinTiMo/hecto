use super::{DocumentStatus, Size, Terminal, UIComponent};
use std::io::Error;

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    size: Size,
}

impl StatusBar {
    pub fn update_status(&mut self, new_status: DocumentStatus) {
        if new_status != self.current_status {
            self.current_status = new_status;
            self.set_needs_redraw(true);
        }
    }
}

impl UIComponent for StatusBar {
    fn set_needs_redraw(&mut self, should_redraw: bool) {
        self.needs_redraw = should_redraw;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), Error> {
        let line_count: String = self.current_status.line_count_to_string();
        let modified_indicator: String = self.current_status.modified_indicator_to_string();

        let beginning: String = format!(
            "{} - {line_count} {modified_indicator}",
            self.current_status.file_name
        );

        let position_indicator: String = self.current_status.position_indicator_to_string();
        let remainder_len: usize = self.size.width.saturating_sub(beginning.len());
        let status: String = format!("{beginning}{position_indicator:>remainder_len$}");

        let to_print: String = if status.len() <= self.size.width {
            status
        } else {
            String::new()
        };

        Terminal::print_inverted_row(origin_row, &to_print)?;

        Ok(())
    }
}
