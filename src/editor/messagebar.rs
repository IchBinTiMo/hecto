use std::io::Error;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redraw: bool
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: &str) {
        if new_message != self.current_message {
            self.current_message = new_message.to_string();
            self.mark_redraw(true);
        }
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, should_redraw: bool) {
        self.needs_redraw = should_redraw;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, _size: Size) {}

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        Terminal::print_row(origin, &self.current_message)
    }
}