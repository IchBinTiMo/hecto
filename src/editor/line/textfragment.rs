use super::GraphemeWidth;
// use crossterm::style::Color;

#[derive(Clone, Debug)]
pub struct TextFragment {
    // pub foreground_color: Option<Color>,
    // pub background_color: Option<Color>,
    // pub annotation_type: Option<AnnotationType>,
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
    pub start_byte_idx: usize,
}