use std::{
    cmp::{max, min},
    fmt::{self, Display},
};
pub use annotationtype::AnnotationType;
use annotation::Annotation;
use annotatedstringpart::AnnotatedStringPart;
use annotatedstringiterator::AnnotatedStringIterator;

pub mod annotationtype;
mod annotation;
mod annotatedstringpart;
mod annotatedstringiterator;

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

    pub fn add_annotation(&mut self, annotation_type: AnnotationType, start_byte_idx: usize, end_byte_idx: usize) {

        self.annotations.push(Annotation {
            annotation_type,
            start_byte_idx,
            end_byte_idx,
        })
    }

    pub fn replace(&mut self, start_byte_idx: usize, end_byte_idx: usize, new_string: &str) {
        let end_byte_idx = min(end_byte_idx, self.string.len());
        self.string.replace_range(start_byte_idx..end_byte_idx, new_string);

        let replace_range_len = end_byte_idx.saturating_sub(start_byte_idx);
        let shortened = new_string.len() < replace_range_len;
        let len_diff = new_string.len().abs_diff(replace_range_len);

        if len_diff == 0 {
            return;
        }

        self.annotations.iter_mut().for_each(|annotation| {
            annotation.start_byte_idx = if annotation.start_byte_idx >= end_byte_idx {
                if shortened {
                    annotation.start_byte_idx.saturating_sub(len_diff)
                } else {
                    annotation.start_byte_idx.saturating_add(len_diff)
                }
            } else if annotation.start_byte_idx >= start_byte_idx {
                if shortened {
                    max(start_byte_idx, annotation.start_byte_idx.saturating_sub(len_diff))
                } else {
                    min(end_byte_idx, annotation.start_byte_idx.saturating_add(len_diff))
                }
            } else {
                annotation.start_byte_idx
            };

            annotation.end_byte_idx = if annotation.end_byte_idx >= end_byte_idx {
                if shortened {
                    annotation.end_byte_idx.saturating_sub(len_diff)
                } else {
                    annotation.end_byte_idx.saturating_add(len_diff)
                }
            } else if annotation.end_byte_idx >= start_byte_idx {
                if shortened {
                    max(start_byte_idx, annotation.end_byte_idx.saturating_sub(len_diff))
                } else {
                    min(end_byte_idx, annotation.end_byte_idx.saturating_add(len_diff))
                }
            } else {
                annotation.end_byte_idx
            }
        });

        self.annotations.retain(|annotation| {
            annotation.start_byte_idx < annotation.end_byte_idx && annotation.start_byte_idx < self.string.len()
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