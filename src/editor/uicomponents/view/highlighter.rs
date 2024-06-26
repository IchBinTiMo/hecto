use super::super::super::{Annotation, AnnotationType, Line};
use crate::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Highlighter<'a> {
    matched_word: Option<&'a str>,
    selected_match: Option<Location>,
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl<'a> Highlighter<'a> {
    pub fn new(matched_word: Option<&'a str>, selected_match: Option<Location>) -> Self {
        Self {
            matched_word,
            selected_match,
            highlights: HashMap::new(),
        }
    }

    pub fn get_annotation(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }

    fn highlight_digits(line: &Line, result: &mut Vec<Annotation>) {
        line.chars().enumerate().for_each(|(idx, ch)| {
            if ch.is_ascii_digit() {
                result.push(Annotation {
                    annotation_type: AnnotationType::Digit,
                    start: idx,
                    end: idx.saturating_add(1),
                });
            }
        });
    }

    fn highlight_matched_words(&self, _line: &Line, result: &mut Vec<Annotation>, search_results: &Option<Vec<GraphemeIdx>>) {
        if let Some(matched_word) = self.matched_word {
            if matched_word.is_empty() {
                return;
            }
        }

        if let Some(search_results) = search_results {
            if let Some(match_word) = self.matched_word {
                for grapheme_idx in search_results {
                    let start = *grapheme_idx;
                    let end = start.saturating_add(match_word.len());
                    result.push(Annotation {
                        annotation_type: AnnotationType::Match,
                        start,
                        end,
                    })
                }
            }
        }
    }

    fn highlight_selected_match(&self, result: &mut Vec<Annotation>) {
        if let Some(seleted_match) = self.selected_match {
            if let Some(match_word) = self.matched_word {
                if match_word.is_empty() {
                    return;
                }

                let start = seleted_match.grapheme_index;
                let end = start.saturating_add(match_word.len());
                result.push(Annotation {
                    annotation_type: AnnotationType::SelectedMatch,
                    start,
                    end,
                })
            }
        }
    }

    pub fn highlight(&mut self, line_idx: LineIdx, line: &Line, search_results: &Option<Vec<GraphemeIdx>>) {
        let mut result = Vec::new();
        Self::highlight_digits(line, &mut result);
        self.highlight_matched_words(line, &mut result, search_results);

        if let Some(selected_match) = self.selected_match {
            if selected_match.line_index == line_idx {
                self.highlight_selected_match(&mut result);
            }
        }

        self.highlights.insert(line_idx, result);
    }
}
