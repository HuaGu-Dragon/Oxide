use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub bytes: Range<usize>,
}
impl Annotation {
    pub fn shift(&mut self, idx: usize) {
        self.bytes.start = self.bytes.start.saturating_add(idx);
        self.bytes.end = self.bytes.end.saturating_add(idx);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
    Number,
    Comment,
    Keyword,
    Type,
    Char,
    Lifetime,
}
