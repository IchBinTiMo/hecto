use std::io::Error;

use super::{
    // terminal::{Size, Terminal},
    Size,
    Terminal,
    UIComponent,
    DocumentStatus
};

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    size: Size,
    // margin_bottom: usize,
    // width: usize,
    // position_y: usize,
    // is_visible: bool,
}

impl StatusBar {
    // pub fn new(margin_bottom: usize) -> Self {
    //     let size = Terminal::size().unwrap_or_default();

    //     let mut status_bar = Self {
    //         current_status: DocumentStatus::default(),
    //         needs_redraw: true,
    //         margin_bottom,
    //         width: size.width,
    //         position_y: size.height.saturating_sub(margin_bottom).saturating_sub(1),
    //         is_visible: false,
    //     };

    //     status_bar.resize(size);

    //     status_bar
    // }

    // pub fn resize(&mut self, size: Size) {
    //     self.width = size.width;
    //     let mut position_y = 0;
    //     let mut is_visible = false;
    //     if let Some(result) = size.height.checked_sub(self.margin_bottom).and_then(|ret| ret.checked_sub(1)) {
    //         position_y = result;
    //         is_visible = true;
    //     }

    //     self.position_y = position_y;
    //     self.is_visible = is_visible;
    //     self.set_needs_redraw(true);
    //     // self.position_y = size.height.saturating_sub(self.margin_bottom).saturating_sub(1);
    // }

    pub fn update_status(&mut self, new_status: DocumentStatus) {
        if new_status != self.current_status {
            self.current_status = new_status;
            self.set_needs_redraw(true);
        }
    }

    // pub fn render(&mut self) {
    //     if !self.needs_redraw || !self.is_visible {
    //         return;
    //     }

    //     if let Ok(size) = Terminal::size() {
    //         let line_count: String = self.current_status.line_count_to_string();
    //         let modified_indicator: String = self.current_status.modified_indicator_to_string();

    //         let beginning: String = format!("{} - {line_count} {modified_indicator}", self.current_status.file_name);

    //         let position_indicator: String = self.current_status.position_indicator_to_string();
    //         let remainder_len: usize = size.width.saturating_sub(beginning.len());
    //         let status: String = format!("{beginning}{position_indicator:>remainder_len$}");

    //         let to_print: String = if status.len() <= size.width {
    //             status
    //         } else {
    //             String::new()
    //         };

    //         let result = Terminal::print_inverted_row(self.position_y, &to_print);

    //         debug_assert!(result.is_ok(), "Failed to render status bar");

    //         self.needs_redraw = false;
    //     }

    //     // let mut status: String = format!("{:?}", self.current_status);

    //     // status.truncate(self.width);

    //     // let result = Terminal::print_row(self.position_y,&status);
    //     // debug_assert!(result.is_ok(), "Failed to render status bar");

    //     // self.needs_redraw = false;
    // }
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

        let beginning: String = format!("{} - {line_count} {modified_indicator}", self.current_status.file_name);

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