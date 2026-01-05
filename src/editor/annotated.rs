use std::{fmt::Display, ops::Range};

use crate::editor::annotated::annotation::{Annotation, AnnotationType};

mod annotation;

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
        let end_idx = std::cmp::min(bytes.end, self.inner.len());
        self.inner.replace_range(bytes, replace_with);
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
