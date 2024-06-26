use super::ByteIdx;
use annotatedstringiterator::AnnotatedStringIterator;
use annotatedstringpart::AnnotatedStringPart;
// use annotation::Annotation;
// pub use annotationtype::AnnotationType;
use super::{Annotation, AnnotationType};
use std::{
    cmp::{max, min},
    fmt::{self, Display},
};

mod annotatedstringiterator;
mod annotatedstringpart;
// mod annotation;
// pub mod annotationtype;

#[derive(Default, Debug)]
pub struct AnnotatedString {
    string: String,
    annotations: Vec<Annotation>,
}

impl AnnotatedString {
    pub fn from(string: &str) -> Self {
        Self {
            string: String::from(string),
            annotations: Vec::new(),
        }
    }

    pub fn add_annotation(
        &mut self,
        annotation_type: AnnotationType,
        start: ByteIdx,
        end: ByteIdx,
    ) {
        self.annotations.push(Annotation {
            annotation_type,
            start,
            end,
        })
    }

    pub fn truncate_left_until(&mut self, until: ByteIdx) {
        self.replace(0, until, "");
    }

    pub fn truncate_right_from(&mut self, from: ByteIdx) {
        self.replace(from, self.string.len(), "");
    }

    pub fn replace(&mut self, start: ByteIdx, end: ByteIdx, new_string: &str) {
        let end = min(end, self.string.len());

        if start > end {
            return;
        }

        self.string.replace_range(start..end, new_string);

        let replace_range_len = end.saturating_sub(start);
        let shortened = new_string.len() < replace_range_len;
        let len_diff = new_string.len().abs_diff(replace_range_len);

        if len_diff == 0 {
            return;
        }

        self.annotations.iter_mut().for_each(|annotation| {
            annotation.start = if annotation.start >= end {
                if shortened {
                    annotation.start.saturating_sub(len_diff)
                } else {
                    annotation.start.saturating_add(len_diff)
                }
            } else if annotation.start >= start {
                if shortened {
                    max(start, annotation.start.saturating_sub(len_diff))
                } else {
                    min(end, annotation.start.saturating_add(len_diff))
                }
            } else {
                annotation.start
            };

            annotation.end = if annotation.end >= end {
                if shortened {
                    annotation.end.saturating_sub(len_diff)
                } else {
                    annotation.end.saturating_add(len_diff)
                }
            } else if annotation.end >= start {
                if shortened {
                    max(start, annotation.end.saturating_sub(len_diff))
                } else {
                    min(end, annotation.end.saturating_add(len_diff))
                }
            } else {
                annotation.end
            }
        });

        self.annotations.retain(|annotation| {
            annotation.start < annotation.end && annotation.start < self.string.len()
        });
    }
}

impl Display for AnnotatedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl<'a> IntoIterator for &'a AnnotatedString {
    type Item = AnnotatedStringPart<'a>;
    type IntoIter = AnnotatedStringIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnnotatedStringIterator {
            annotated_string: self,
            current_idx: 0,
        }
    }
}
