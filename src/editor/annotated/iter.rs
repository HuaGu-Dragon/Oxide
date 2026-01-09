use crate::editor::annotated::{AnnotatedString, part::Part};

pub struct IntoIter<'a> {
    inner: &'a AnnotatedString,
    idx: usize,
}

impl<'a> IntoIter<'a> {
    pub fn new(inner: &'a AnnotatedString) -> Self {
        Self { inner, idx: 0 }
    }
}

impl<'a> Iterator for IntoIter<'a> {
    type Item = Part<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.inner.len() {
            return None;
        }

        if let Some(annotation) = self.inner.annotations.iter().rfind(|annotation| {
            annotation.bytes.start <= self.idx && annotation.bytes.end > self.idx
        }) {
            let end_idx = std::cmp::min(annotation.bytes.end, self.inner.inner.len());
            let start_idx = self.idx;

            self.idx = end_idx;

            return Some(Part {
                inner: &self.inner.inner[start_idx..end_idx],
                annotation: Some(annotation.annotation_type),
            });
        }

        let mut end_idx = self.inner.inner.len();
        for annotation in &self.inner.annotations {
            if annotation.bytes.start > self.idx && annotation.bytes.start < end_idx {
                end_idx = annotation.bytes.start;
            }
        }

        let start_idx = self.idx;
        self.idx = end_idx;

        Some(Part {
            inner: &self.inner.inner[start_idx..end_idx],
            annotation: None,
        })
    }
}
