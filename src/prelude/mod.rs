pub type GraphemeIdx = usize;
pub type LineIdx = usize;
pub type ByteIdx = usize;
pub type ColIdx = usize;
pub type RowIdx = usize;

pub use location::Location;
pub use position::Position;
pub use size::Size;

mod location;
mod position;
mod size;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
