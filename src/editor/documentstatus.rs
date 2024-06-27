use super::FileType;
use crate::prelude::*;

#[derive(Default, Eq, PartialEq, Debug)]
pub struct DocumentStatus {
    pub current_grapheme_index: GraphemeIdx,
    pub current_line_index: LineIdx,
    pub file_name: String,
    pub file_type: FileType,
    pub is_modified: bool,
    pub total_lines: usize,
}

impl DocumentStatus {
    pub fn modified_indicator_to_string(&self) -> String {
        if self.is_modified {
            String::from("(modified)")
        } else {
            String::new()
        }
    }

    pub fn line_count_to_string(&self) -> String {
        format!("{} lines", self.total_lines)
    }

    pub fn position_indicator_to_string(&self) -> String {
        format!(
            "{}:{}",
            self.current_line_index.saturating_add(1),
            self.current_grapheme_index.saturating_add(1)
        )
    }

    pub fn file_type_to_string(&self) -> String {
        format!("{}", self.file_type)
    }
}
