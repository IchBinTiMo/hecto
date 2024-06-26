use super::AnnotatedString;
use crate::prelude::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{
        Attribute::{Reset, Reverse},
        Print, ResetColor, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    Command,
};
use std::io::{stdout, Error, Write};

use attribute::Attribute;

mod attribute;

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation
/// only spans over at most `usize::MAX` or `u16::size` rows/columns, whichever is smaller.
/// Each size returned truncates to min(`usize::MAX`, `u16::MAX`)
/// And should you attempt to set the caret out of these bounds, it will also be truncated.
pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }

    pub fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Move the caret to the given position
    /// # Arguments
    /// * `position` - The position to move the caret to. Will be truncated to `u16::MAX` if out of bounds
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    pub fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(EnableLineWrap)?;
        Ok(())
    }

    pub fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(DisableLineWrap)?;
        Ok(())
    }

    pub fn set_title(title: &str) -> Result<(), Error> {
        Self::queue_command(SetTitle(title))?;
        Ok(())
    }

    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }

    /// Returns the current size of the Terminal
    /// Edge Case for platforms where `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize ` < `z` < `u16`
    pub fn size() -> Result<Size, Error> {
        let (width_16, height_16) = size()?;

        #[allow(clippy::as_conversions)]
        let width: usize = width_16 as usize;
        let height: usize = height_16 as usize;

        Ok(Size { width, height })
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn print_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(Position { row, col: 0 })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }

    pub fn print_annotated_row(
        row: RowIdx,
        annotated_string: &AnnotatedString,
    ) -> Result<(), Error> {
        Self::move_caret_to(Position { row, col: 0 })?;
        Self::clear_line()?;

        annotated_string
            .into_iter()
            .try_for_each(|part| -> Result<(), Error> {
                if let Some(annotation_type) = part.annotation_type {
                    let attribute = annotation_type.into();
                    Self::set_attribute(&attribute)?;
                }

                Self::print(part.string)?;
                Self::reset_color()?;
                Ok(())
            })?;
        Ok(())
    }

    fn set_attribute(attribute: &Attribute) -> Result<(), Error> {
        if let Some(foreground_color) = attribute.foreground {
            Self::queue_command(SetForegroundColor(foreground_color))?;
        }

        if let Some(background_color) = attribute.background {
            Self::queue_command(SetBackgroundColor(background_color))?;
        }

        Ok(())
    }

    fn reset_color() -> Result<(), Error> {
        Self::queue_command(ResetColor)?;
        Ok(())
    }

    pub fn print_inverted_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        let width = Self::size()?.width;
        Self::print_row(
            row,
            &format!("{}{:width$.width$}{}", Reverse, line_text, Reset),
        )
    }
}
