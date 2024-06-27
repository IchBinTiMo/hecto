use super::{AnnotatedString, Annotation};
use crate::prelude::*;
use graphemewidth::GraphemeWidth;
use std::{
    // cmp::min,
    fmt::{self, Display},
    ops::{Deref, Range},
};
use textfragment::TextFragment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod graphemewidth;
mod textfragment;

// type GraphemeIdx = usize;
// type ByteIdx = usize;
// type ColIdx = usize;

#[derive(Default, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments: Vec<TextFragment> = Self::str_to_fragments(line_str);

        Self {
            fragments,
            string: String::from(line_str),
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true)
            .map(|(byte_idx, grapheme)| {
                let (replacement, rendered_width) = Self::get_replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width: usize = grapheme.width();
                            let rendered_width: GraphemeWidth = match unicode_width {
                                0 | 1 => GraphemeWidth::Half,
                                _ => GraphemeWidth::Full,
                            };
                            (None, rendered_width)
                        },
                        |replacement| (Some(replacement), GraphemeWidth::Half),
                    );

                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                    start: byte_idx,
                }
            })
            .collect()
    }

    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width: usize = for_str.width();

        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars: std::str::Chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        self.get_annotated_visible_substr(range, None).to_string()
    }

    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIdx>,
        annotations: Option<&Vec<Annotation>>,
    ) -> AnnotatedString {
        if range.start >= range.end {
            return AnnotatedString::default();
        }
        // Create a new annotated string
        let mut result = AnnotatedString::from(&self.string);

        if let Some(annotations) = annotations {
            for annotation in annotations {
                result.add_annotation(annotation.annotation_type, annotation.start, annotation.end);
            }
        }

        // Insert replacement characters and truncate if needed
        // and we iterate fragments from rightmost column first
        let mut fragment_start = self.width();

        for fragment in self.fragments.iter().rev() {
            let fragment_end = fragment_start;

            fragment_start = fragment_start.saturating_sub(fragment.rendered_width.into());

            if fragment_start > range.end {
                continue; // This fragment is not visible
            }

            if fragment_start < range.end && fragment_end > range.end {
                // the fragment is cut into two parts,
                // the left part is visible,
                // and the right part is not visible
                result.replace(fragment.start, self.string.len(), "...");
                continue;
            } else if fragment_start == range.end {
                result.truncate_right_from(fragment.start);
                continue;
            }

            if fragment_end <= range.start {
                // the fragment ends at the start of the range
                result.truncate_left_until(fragment.start.saturating_add(fragment.grapheme.len()));
                break; // break out of the loop since all fragments remained are not visible
            } else if fragment_start < range.start && fragment_end > range.start {
                // the fragment is cut into two parts,
                // the right part is visible,
                // and the left part is not visible
                result.replace(
                    0,
                    fragment.start.saturating_add(fragment.grapheme.len()),
                    "...",
                );
                break; // break out of the loop since all fragments remained are not visible
            }

            if fragment_start >= range.start && fragment_end <= range.end {
                // the fragment is completely visible
                if let Some(replcement) = fragment.replacement {
                    let start = fragment.start;
                    let end = start.saturating_add(fragment.grapheme.len());

                    result.replace(start, end, &replcement.to_string());
                }
            }
        }

        result
    }

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> ColIdx {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn width(&self) -> ColIdx {
        self.width_until(self.grapheme_count())
    }

    pub fn insert_char(&mut self, character: char, at: usize) {
        if let Some(fragment) = self.fragments.get_mut(at) {
            self.string.insert(fragment.start, character);
        } else {
            self.string.push(character);
        }

        self.rebuild_fragments();
    }

    pub fn delete_char(&mut self, at: usize) {
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start;
            let end = fragment.start.saturating_add(fragment.grapheme.len());
            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.to_string());
        self.rebuild_fragments();
    }

    pub fn split(&mut self, at: usize) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start);

            self.rebuild_fragments();

            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
        if byte_idx > self.string.len() {
            return None;
        }

        self.fragments
            .iter()
            .position(|fragment| fragment.start >= byte_idx)
    }

    pub fn search(&self, query: &str) -> Option<Vec<ByteIdx>> {
        let result: Vec<ByteIdx> = self
            .string
            .match_indices(query)
            .map(|(byte_idx, _)| self.byte_idx_to_grapheme_idx(byte_idx))
            .map(|x| x.unwrap())
            .collect();

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
