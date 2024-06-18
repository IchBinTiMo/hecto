use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;

use super::terminal::Size;

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}
pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Insert(char),
    Backspace,
    Delete,
    Quit,
}

// clippy::as_conversions: Will run into problems for rare edge case systems where usize < u16
#[allow(clippy::as_conversions)]
impl TryFrom<Event> for EditorCommand {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char(character), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(character)),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Delete, _) => Ok(Self::Delete),
                _ => Err(format!("Unsupported event: {:?}", event)),
            },
            Event::Resize(width_16, height_16) => Ok(Self::Resize(Size { width: width_16 as usize, height: height_16 as usize })),
            _ => Err(format!("Unsupported event: {:?}", event)),
        }
    }
}