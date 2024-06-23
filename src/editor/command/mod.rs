use super::Size;

use std::convert::TryFrom;
use crossterm::event::Event;

pub use edit::Edit;
pub use movecommand::Move;
pub use system::System;

mod edit;
mod movecommand;
mod system;

#[derive(Clone, Copy)]
pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

#[allow(clippy::as_conversions)]
impl TryFrom<Event> for Command {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key_event) => Edit::try_from(key_event)
                .map(Command::Edit)
                .or_else(|_| Move::try_from(key_event).map(Command::Move))
                .or_else(|_| System::try_from(key_event).map(Command::System))
                .map_err(|_err| format!("Event not supported: {key_event:?}")),
            Event::Resize(width_u16, height_u16) => Ok(Self::System(System::Resize(Size {
                width: width_u16 as usize,
                height: height_u16 as usize,
            }))),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}