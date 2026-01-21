use crate::editor::{annotated::annotation::Annotation, view::line::Line};

pub trait SyntaxHighlighter {
    fn highlight(&mut self, idx: usize, line: &Line);
    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>>;
}
