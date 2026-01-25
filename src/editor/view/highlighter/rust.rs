use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{highlighter::syntax_highlight::SyntaxHighlighter, line::Line},
};

const KEYWORDS: [&str; 52] = [
    "Self",
    "abstract",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "macro_rules",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
];

const TYPES: [&str; 23] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "Option", "Result", "String", "str", "Vec", "HashMap", "VecDeque",
];

#[derive(Default)]
pub struct RustHighlighter {
    highlights: HashMap<usize, Vec<Annotation>>,
}

fn is_numeric_literal(input: &str) -> bool {
    if input.len() < 3 {
        return false;
    }

    let mut chars = input.chars();
    if chars.next() != Some('0') {
        return false;
    }

    let base = match chars.next() {
        Some('b' | 'B') => 2,
        Some('o' | 'O') => 8,
        Some('x' | 'X') => 16,
        _ => return false,
    };

    chars.all(|c| c.is_digit(base))
}

fn is_valid_number(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    if is_numeric_literal(input) {
        return true;
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
    fn highlight(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        for (idx, word) in line.split_word_bound_indices() {
            let mut add_annotation = |ty| {
                res.push(Annotation {
                    annotation_type: ty,
                    bytes: idx..idx.saturating_add(word.len()),
                });
            };

            if is_valid_number(word) {
                add_annotation(AnnotationType::Number);
            } else if KEYWORDS.contains(&word) {
                add_annotation(AnnotationType::Keyword);
            } else if TYPES.contains(&word) {
                add_annotation(AnnotationType::Type);
            }
        }
    }
}

impl SyntaxHighlighter for RustHighlighter {
    fn highlight(&mut self, idx: usize, line: &crate::editor::view::line::Line) {
        let mut res = vec![];
        self.highlight(line, &mut res);
        self.highlights.insert(idx, res);
    }

    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
