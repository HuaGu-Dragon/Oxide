use std::ops::Range;

#[derive(Debug)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub bytes: Range<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
    Digit
}
