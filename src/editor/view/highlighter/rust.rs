use std::collections::HashMap;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{highlighter::syntax_highlight::SyntaxHighlighter, line::Line},
};

#[derive(Default)]
pub struct RustHighlighter {
    highlights: HashMap<usize, Vec<Annotation>>,
}

impl RustHighlighter {
    fn highlight_digits(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        line.chars().enumerate().for_each(|(idx, ch)| {
            if ch.is_ascii_digit() {
                res.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    bytes: idx..idx.saturating_add(1),
                });
            }
        });
    }
}

impl SyntaxHighlighter for RustHighlighter {
    fn highlight(&mut self, idx: usize, line: &crate::editor::view::line::Line) {
        let mut res = vec![];
        self.highlight_digits(line, &mut res);
        self.highlights.insert(idx, res);
    }

    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
