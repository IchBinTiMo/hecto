use super::GraphemeWidth;


pub struct TextFragment {
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
    pub start_byte_idx: usize,
}