use super::{Annotation, AnnotationType, Line, SyntaxHighlighter};
use crate::prelude::*;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

const KEYWORDS: [&str; 52] = ["break",
"const",
"continue",
"crate",
"else",
"enum",
"extern",
"false",
"fn",
"for",
"if",
"impl",
"in",
"let",
"loop",
"match",
"mod",
"move",
"mut",
"pub",
"ref",
"return",
"self",
"Self",
"static",
"struct",
"super",
"trait",
"true",
"type",
"unsafe",
"use",
"where",
"while",
"async",
"await",
"dyn",
"abstract",
"become",
"box",
"do",
"final",
"macro",
"override",
"priv",
"typeof",
"unsized",
"virtual",
"yield",
"try",
"macro_rules",
"union",];

const TYPES: [&str; 22] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "Option", "Result", "String", "str", "Vec", "HashMap",
];

const KNOWN_VALUES: [&str; 6] = ["Some", "None", "true", "false", "Ok", "Err"];

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

fn is_valid_number(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    if is_numeric_literal(word) {
        return true;
    }

    let mut chars = word.chars();

    if let Some(first_char) = chars.next() {
        if !first_char.is_ascii_digit() {
            return false;
        }
    }

    let mut has_dot = false;
    let mut has_e = false;
    let mut prev_was_digit = true;


    for ch in chars {
        match ch {
            '0'..='9' => {
                prev_was_digit = true;
            }
            '_' => {
                if !prev_was_digit {
                    return false;
                }

                prev_was_digit = false;
            }
            '.' => {
                // if prev_was_digit && !has_dot {
                //     has_dot = true;
                //     prev_was_digit = false;
                // } else {
                //     return false;
                // }
                if has_dot || has_e || !prev_was_digit {
                    return false;
                }

                has_dot = true;
                prev_was_digit = false;
            }
            'e' | 'E' => {
                // if prev_was_digit && !has_e {
                //     has_e = true;
                //     prev_was_digit = false;
                // } else {}
                if has_e || !prev_was_digit {
                    return false;
                }

                has_e = true;
                prev_was_digit = false;
            }
            _ => return false
        }
    }

    prev_was_digit
}

fn is_numeric_literal(word: &str) -> bool {
    if word.len() < 3 {
        return false;
    }

    let mut chars = word.chars();

    if chars.next() != Some('0') {
        return false;
    }

    let base = match chars.next() {
        Some('b' | 'B') => 2,
        Some('o' | 'O') => 8,
        Some('x' | 'X') => 16,
        _ => return false,
    };

    chars.all(|ch| ch.is_digit(base))
}

fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

fn is_type(word: &str) -> bool {
    TYPES.contains(&word)
}

fn is_known_value(word: &str) -> bool {
    KNOWN_VALUES.contains(&word)
}
// #[derive(Default)]
// pub struct Highlighter<'a> {
//     matched_word: Option<&'a str>,
//     selected_match: Option<Location>,
//     highlights: HashMap<LineIdx, Vec<Annotation>>,
// }

// impl RustSyntaxHighlighter {
//     // pub fn new(matched_word: Option<&'a str>, selected_match: Option<Location>) -> Self {
//     //     Self {
//     //         matched_word,
//     //         selected_match,
//     //         highlights: HashMap::new(),
//     //     }
//     // }

//     // pub fn get_annotation(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
//     //     self.highlights.get(&idx)
//     // }

//     fn highlight_digits(line: &Line, result: &mut Vec<Annotation>) {
//         line.chars().enumerate().for_each(|(idx, ch)| {
//             if ch.is_ascii_digit() {
//                 result.push(Annotation {
//                     annotation_type: AnnotationType::Number,
//                     start: idx,
//                     end: idx.saturating_add(1),
//                 });
//             }
//         });
//     }

//     // fn highlight_matched_words(&self, _line: &Line, result: &mut Vec<Annotation>, search_results: &Option<Vec<GraphemeIdx>>) {
//     //     if let Some(matched_word) = self.matched_word {
//     //         if matched_word.is_empty() {
//     //             return;
//     //         }
//     //     }

//     //     if let Some(search_results) = search_results {
//     //         if let Some(match_word) = self.matched_word {
//     //             for grapheme_idx in search_results {
//     //                 let start = *grapheme_idx;
//     //                 let end = start.saturating_add(match_word.len());
//     //                 result.push(Annotation {
//     //                     annotation_type: AnnotationType::Match,
//     //                     start,
//     //                     end,
//     //                 })
//     //             }
//     //         }
//     //     }
//     // }

//     // fn highlight_selected_match(&self, result: &mut Vec<Annotation>) {
//     //     if let Some(seleted_match) = self.selected_match {
//     //         if let Some(match_word) = self.matched_word {
//     //             if match_word.is_empty() {
//     //                 return;
//     //             }

//     //             let start = seleted_match.grapheme_index;
//     //             let end = start.saturating_add(match_word.len());
//     //             result.push(Annotation {
//     //                 annotation_type: AnnotationType::SelectedMatch,
//     //                 start,
//     //                 end,
//     //             })
//     //         }
//     //     }
//     // }

//     // pub fn highlight(&mut self, line_idx: LineIdx, line: &Line, search_results: &Option<Vec<GraphemeIdx>>) {
//     //     let mut result = Vec::new();
//     //     Self::highlight_digits(line, &mut result);
//     //     self.highlight_matched_words(line, &mut result, search_results);

//     //     if let Some(selected_match) = self.selected_match {
//     //         if selected_match.line_index == line_idx {
//     //             self.highlight_selected_match(&mut result);
//     //         }
//     //     }

//     //     self.highlights.insert(line_idx, result);
//     // }
// }

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(
        &mut self,
        line_idx: LineIdx,
        line: &Line,
        _search_results: &Option<Vec<GraphemeIdx>>,
    ) {
        let mut result = Vec::new();
        // Self::highlight_digits(line, &mut result);
        for (start_idx, word) in line.split_word_bound_indices() {
            let mut annotation_type = None;
            if is_valid_number(word) {
                annotation_type = Some(AnnotationType::Number);
            } else if is_keyword(word) {
                annotation_type = Some(AnnotationType::Keyword);
            } else if is_type(word) {
                annotation_type = Some(AnnotationType::Type);
            } else if is_known_value(word) {
                annotation_type = Some(AnnotationType::KnownValue);
            }

            if let Some(annotation_type) = annotation_type {
                result.push(Annotation {
                    annotation_type,
                    start: start_idx,
                    end: start_idx.saturating_add(word.len()),
                });
                
            }
        }

        
        self.highlights.insert(line_idx, result);
    }

    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}
