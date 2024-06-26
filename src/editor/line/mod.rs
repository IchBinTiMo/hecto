use std::{
    // cmp::min,
    fmt::{self, Display},
    ops::{Deref, Range},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use graphemewidth::GraphemeWidth;
use textfragment::TextFragment;
use super::{AnnotatedString, AnnotationType};
use crate::prelude::*;



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
        self.get_annotated_visible_substr(range, None, None, &None)
            .to_string()
    }

    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIdx>,
        query: Option<&str>,
        selected_match: Option<GraphemeIdx>,
        search_results: &Option<Vec<GraphemeIdx>>,
    ) -> AnnotatedString {
        if range.start >= range.end {
            return AnnotatedString::default();
        }
        // Create a new annotated string
        let mut result = AnnotatedString::from(&self.string);

        // Highlight Digits
        self.string.chars().enumerate().for_each(|(idx, ch)| {
            if ch.is_ascii_digit() {
                result.add_annotation(AnnotationType::Digit, idx, idx.saturating_add(1));
            }
        });


        if let Some(query) = query {
            if !query.is_empty() {
                if let Some(search_results) = search_results {
                    for grapheme_idx in search_results {
                        let start_byte_idx = self.grapheme_idx_to_byte_idx(*grapheme_idx);
                        let end_byte_idx = start_byte_idx.saturating_add(query.len());
                        if let Some(seleted_match) = selected_match {
                            if *grapheme_idx == seleted_match {
                                result.add_annotation(
                                    AnnotationType::SelectedMatch,
                                    start_byte_idx,
                                    end_byte_idx,
                                );
                            } else {
                                result.add_annotation(
                                    AnnotationType::Match,
                                    start_byte_idx,
                                    end_byte_idx,
                                );
                            }
                        } else {
                            result.add_annotation(
                                AnnotationType::Match,
                                start_byte_idx,
                                end_byte_idx,
                            );
                        }
                    }
                }
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
                // result.replace(fragment.start, self.string.len(), "");
                result.truncate_right_from(fragment.start);
                continue;
            }

            if fragment_end <= range.start {
                // the fragment ends at the start of the range
                result.truncate_left_until(fragment.start.saturating_add(fragment.grapheme.len()));
                // result.replace(
                //     0,
                //     fragment
                //         .start
                //         .saturating_add(fragment.grapheme.len()),
                //     "",
                // );
                break; // break out of the loop since all fragments remained are not visible
            } else if fragment_start < range.start && fragment_end > range.start {
                // the fragment is cut into two parts,
                // the right part is visible,
                // and the left part is not visible
                result.replace(
                    0,
                    fragment.start.saturating_add(fragment.grapheme.len()),
                    // fragment
                    //     .start
                    //     .saturating_add(fragment.grapheme.len()),
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
            let end = fragment
                .start
                .saturating_add(fragment.grapheme.len());
            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    pub fn append(&mut self, other: &Self) {
        let mut concat = self.to_string();

        concat.push_str(&other.to_string());
        self.fragments = Self::str_to_fragments(&concat);
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

    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        if grapheme_idx == 0 || self.grapheme_count() == 0 {
            return 0;
        }

        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!("Grapheme index {:?} out of range", grapheme_idx);
                }

                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.start,
        )
    }

    pub fn search(&self, query: &str) -> Option<Vec<GraphemeIdx>> {
        let result: Vec<GraphemeIdx> = self
            .string
            .match_indices(query)
            // .map(|byte_idx, _ | self.match_grapheme_clusters(&matches, query))
            // .map(|_, grapheme_idx| grapheme_idx)
            .map(|(byte_idx, _)| self.byte_idx_to_grapheme_idx(byte_idx))
            .map(|x| x.unwrap())
            .collect();

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    // fn match_grapheme_clusters(&self, matches: &[ByteIdx], query: &str) -> Vec<(ByteIdx, GraphemeIdx)> {
    //     let grapheme_count = query.graphemes(true).count();

    //     matches.iter().filter_map(|&start| {
    //         self.byte_idx_to_grapheme_idx(start).and_then(|grapheme_idx| {
    //             self.fragments.get(grapheme_idx..grapheme_idx.saturating_add(grapheme_count)).and_then(|fragments| {
    //                 let substring = fragments.iter().map(|fragment| fragment.grapheme.as_str()).collect::<String>();

    //                 (substring == query).then_some((start, grapheme_idx))
    //             })
    //         })
    //     }).collect()
    // }
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
