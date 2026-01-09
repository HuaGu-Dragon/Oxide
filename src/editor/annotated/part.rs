use crate::editor::annotated::annotation::AnnotationType;

pub struct Part<'a> {
    pub inner: &'a str,
    pub annotation: Option<AnnotationType>,
}
