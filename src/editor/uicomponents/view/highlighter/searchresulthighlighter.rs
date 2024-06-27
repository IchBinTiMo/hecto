use super::{syntaxhighlighter::SyntaxHighlighter, Annotation, AnnotationType, Line};
use crate::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchResultHighlighter<'a> {
    matched_word: &'a str,
    selected_match: Option<Location>,
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl<'a> SearchResultHighlighter<'a> {
    pub fn new(matched_word: &'a str, selected_match: Option<Location>) -> Self {
        Self {
            matched_word,
            selected_match,
            highlights: HashMap::new(),
        }
    }

    fn highlight_matched_words(
        &self,
        _line: &Line,
        result: &mut Vec<Annotation>,
        search_results: &Option<Vec<GraphemeIdx>>,
    ) {
        if self.matched_word.is_empty() {
            return;
        }

        if let Some(search_results) = search_results {
            for grapheme_idx in search_results {
                let start = *grapheme_idx;
                let end = start.saturating_add(self.matched_word.len());
                result.push(Annotation {
                    annotation_type: AnnotationType::Match,
                    start,
                    end,
                })
            }
        }
    }

    fn highlight_selected_match(&self, result: &mut Vec<Annotation>) {
        if let Some(seleted_match) = self.selected_match {
            if self.matched_word.is_empty() {
                return;
            }

            let start = seleted_match.grapheme_index;
            let end = start.saturating_add(self.matched_word.len());
            result.push(Annotation {
                annotation_type: AnnotationType::SelectedMatch,
                start,
                end,
            })
        }
    }
}

impl<'a> SyntaxHighlighter for SearchResultHighlighter<'a> {
    fn highlight(
        &mut self,
        line_idx: LineIdx,
        line: &Line,
        search_results: &Option<Vec<GraphemeIdx>>,
    ) {
        let mut result = Vec::new();
        self.highlight_matched_words(line, &mut result, search_results);

        if let Some(selected_match) = self.selected_match {
            if selected_match.line_index == line_idx {
                self.highlight_selected_match(&mut result);
            }
        }

        self.highlights.insert(line_idx, result);
    }

    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}
