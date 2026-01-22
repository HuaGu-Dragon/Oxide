use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{highlighter::syntax_highlight::SyntaxHighlighter, line::Line},
};

#[derive(Default)]
pub struct RustHighlighter {
    highlights: HashMap<usize, Vec<Annotation>>,
}

fn is_valid_number(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    let mut chars = input.chars();

    if let Some(first_char) = chars.next()
        && !first_char.is_ascii_digit()
    {
        return false;
    }

    let mut seen_dot = false;
    let mut seen_e = false;
    let mut prev_was_digit = true;

    for char in chars {
        match char {
            '0'..='9' => {
                prev_was_digit = true;
            }
            '_' => {
                if !prev_was_digit {
                    return false;
                }
                prev_was_digit = false;
            }
            '.' => {
                if seen_dot || seen_e || !prev_was_digit {
                    return false;
                }
                seen_dot = true;
                prev_was_digit = false;
            }
            'e' | 'E' => {
                if seen_e || !prev_was_digit {
                    return false;
                }
                seen_e = true;
                prev_was_digit = false;
            }
            _ => {
                return false;
            }
        }
    }

    prev_was_digit
}

impl RustHighlighter {
    fn highlight_number(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        for (idx, word) in line.split_word_bound_indices() {
            if is_valid_number(word) {
                res.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    bytes: idx..idx.saturating_add(word.len()),
                });
            }
        }
    }
}

impl SyntaxHighlighter for RustHighlighter {
    fn highlight(&mut self, idx: usize, line: &crate::editor::view::line::Line) {
        let mut res = vec![];
        self.highlight_number(line, &mut res);
        self.highlights.insert(idx, res);
    }

    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
