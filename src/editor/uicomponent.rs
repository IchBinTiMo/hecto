use std::io::Error;

use super::Size;

pub trait UIComponent {
    // marks the component as needing to be redrawn
    fn set_needs_redraw(&mut self, value: bool);

    // Determines if the component needs to be redrawn
    fn needs_redraw(&self) -> bool;

    // Update the size andmarks as redraw needed
    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.set_needs_redraw(true);
    }

    // Update the size.
    // Need to be implemented by each UIComponent
    fn set_size(&mut self, size: Size);

    // Draw the component if it's visible and in need of redrawing
    fn render(&mut self, origin_row: usize) {
        if self.needs_redraw() {
            match self.draw(origin_row) {
                Ok(()) => self.set_needs_redraw(false),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not render component: {err:?}");
                    }
                },
            }
        }
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), Error>;

}