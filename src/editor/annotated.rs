use std::{fmt::Display, ops::Range};

use crate::editor::annotated::annotation::{Annotation, AnnotationType};

pub mod annotation;

#[derive(Debug, Default)]
pub struct AnnotatedString {
    inner: String,
    annotations: Vec<Annotation>,
}

impl AnnotatedString {
    pub fn add_annotation(&mut self, annotation_type: AnnotationType, bytes: Range<usize>) {
        self.annotations.push(Annotation {
            annotation_type,
            bytes,
        });
    }

    pub fn replace(&mut self, bytes: Range<usize>, replace_with: &str) {
        if bytes.start > self.inner.len() {
            return;
        }
        let start_idx = bytes.start;
        let end_idx = std::cmp::min(bytes.end, self.inner.len());
        let replaced_range_len = end_idx.saturating_sub(bytes.start);
        self.inner.replace_range(bytes, replace_with);

        let shortened = replace_with.len() < replaced_range_len;
        let difference = replace_with.len().abs_diff(replaced_range_len);

        if difference == 0 {
            return;
        }

        self.annotations.iter_mut().for_each(|annotation| {
            annotation.bytes.start = if annotation.bytes.start >= end_idx {
                if shortened {
                    annotation.bytes.start.saturating_sub(difference)
                } else {
                    annotation.bytes.start.saturating_add(difference)
                }
            } else if annotation.bytes.start >= start_idx {
                if shortened {
                    std::cmp::max(start_idx, annotation.bytes.start.saturating_sub(difference))
                } else {
                    std::cmp::min(end_idx, annotation.bytes.start.saturating_add(difference))
                }
            } else {
                annotation.bytes.start
            };
            annotation.bytes.end = if annotation.bytes.end >= end_idx {
                if shortened {
                    annotation.bytes.end.saturating_sub(difference)
                } else {
                    annotation.bytes.end.saturating_add(difference)
                }
            } else if annotation.bytes.end >= start_idx {
                if shortened {
                    std::cmp::max(start_idx, annotation.bytes.end.saturating_sub(difference))
                } else {
                    std::cmp::min(end_idx, annotation.bytes.end.saturating_add(difference))
                }
            } else {
                annotation.bytes.end
            }
        });

        self.annotations.retain(|annotation| {
            annotation.bytes.start < annotation.bytes.end
                && annotation.bytes.start < self.inner.len()
        });
    }
}

impl From<&str> for AnnotatedString {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for AnnotatedString {
    fn from(value: String) -> Self {
        Self {
            inner: value,
            ..Default::default()
        }
    }
}

impl Display for AnnotatedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
