use crate::editor::{Line, Position};

use super::Location;

pub struct SearchInfo {
    pub current_idx: Option<usize>,
    pub prev_location: Location,
    pub prev_scroll_offset: Position,
    pub query: Option<Line>,
    pub result: Option<Vec<Location>>
}