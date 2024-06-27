use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FileType {
    Rust,
    #[default]
    Text,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Text => write!(f, "Text"),
        }
    }
}
