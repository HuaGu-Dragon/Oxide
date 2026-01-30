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
    highlights: Vec<Vec<Annotation>>,
    multiple_lines_comment: usize,
    multiple_str: bool,
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
    fn initial_annotation(&mut self, line: &Line) -> Option<Annotation> {
        if self.multiple_str {
            self.annotate_multiple_str(line)
        } else if self.multiple_lines_comment > 0 {
            self.annotate_multiple_comment(line)
        } else {
            None
        }
    }

    fn highlight(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        let mut input = line.split_word_bound_indices().peekable();

        if let Some(annotation) = self.initial_annotation(line) {
            while let Some((next_idx, _)) = input.peek() {
                if next_idx >= &annotation.bytes.end {
                    break;
                }
                input.next();
            }

            res.push(annotation);
        }
        while let Some((idx, _)) = input.next() {
            let remainder = &line[idx..];

            if let Some(mut annotation) = self
                .annotate_multiple_comment(remainder)
                .or_else(|| self.annotate_multiple_str(remainder))
                .or_else(|| annotate_comment(remainder))
                .or_else(|| annotate_char(remainder))
                .or_else(|| annotate_lifetime(remainder))
                .or_else(|| annotate_keyword(remainder))
                .or_else(|| annotate_number(remainder))
                .or_else(|| annotate_type(remainder))
            {
                annotation.shift(idx);

                while let Some((next_idx, _)) = input.peek() {
                    if next_idx >= &annotation.bytes.end {
                        break;
                    }
                    input.next();
                }

                res.push(annotation);
            }
        }
    }

    fn annotate_multiple_str(&mut self, input: &str) -> Option<Annotation> {
        let mut chars = input.char_indices();
        while let Some((idx, c)) = chars.next() {
            if c == '\\' && self.multiple_str {
                chars.next();
                continue;
            }
            if c == '"' {
                if self.multiple_str {
                    self.multiple_str = false;
                    return Some(Annotation {
                        annotation_type: AnnotationType::String,
                        bytes: 0..idx.saturating_add(1),
                    });
                }
                self.multiple_str = true;
            }

            if !self.multiple_str {
                return None;
            }
        }

        self.multiple_str.then_some(Annotation {
            annotation_type: AnnotationType::String,
            bytes: 0..input.len(),
        })
    }

    fn annotate_multiple_comment(&mut self, input: &str) -> Option<Annotation> {
        let mut chars = input.char_indices().peekable();

        while let Some((_, c)) = chars.next() {
            if c == '/'
                && let Some((_, '*')) = chars.peek()
            {
                self.multiple_lines_comment = self.multiple_lines_comment.saturating_add(1);
                chars.next();
            } else if self.multiple_lines_comment == 0 {
                return None;
            } else if c == '*'
                && let Some((idx, '/')) = chars.peek()
            {
                self.multiple_lines_comment = self.multiple_lines_comment.saturating_sub(1);

                if self.multiple_lines_comment == 0 {
                    return Some(Annotation {
                        annotation_type: AnnotationType::Comment,
                        bytes: 0..idx.saturating_add(1),
                    });
                }

                chars.next();
            }
        }

        (self.multiple_lines_comment > 0).then_some(Annotation {
            annotation_type: AnnotationType::Comment,
            bytes: 0..input.len(),
        })
    }
}

fn annotate_next_word(
    input: &str,
    ty: AnnotationType,
    is_valid: impl Fn(&str) -> bool,
) -> Option<Annotation> {
    if let Some(word) = input.split_word_bounds().next()
        && is_valid(word)
    {
        return Some(Annotation {
            annotation_type: ty,
            bytes: 0..word.len(),
        });
    }

    None
}

fn annotate_number(input: &str) -> Option<Annotation> {
    annotate_next_word(input, AnnotationType::Number, is_valid_number)
}

fn annotate_keyword(input: &str) -> Option<Annotation> {
    annotate_next_word(input, AnnotationType::Type, |word| KEYWORDS.contains(&word))
}

fn annotate_type(input: &str) -> Option<Annotation> {
    annotate_next_word(input, AnnotationType::Type, |word| TYPES.contains(&word))
}

fn annotate_lifetime(input: &str) -> Option<Annotation> {
    let mut iter = input.split_word_bound_indices();

    if let Some((_, "'")) = iter.next()
        && let Some((idx, lifetime)) = iter.next()
    {
        return Some(Annotation {
            annotation_type: AnnotationType::Lifetime,
            bytes: 0..idx.saturating_add(lifetime.len()),
        });
    }

    None
}

fn annotate_char(input: &str) -> Option<Annotation> {
    let mut iter = input.split_word_bound_indices().peekable();

    if let Some((_, "\'")) = iter.next() {
        if let Some((_, "\\")) = iter.peek() {
            iter.next();
        }
        iter.next();

        if let Some((idx, "\'")) = iter.next() {
            return Some(Annotation {
                annotation_type: AnnotationType::Char,
                bytes: 0..idx.saturating_add(1),
            });
        }
    }
    None
}

fn annotate_comment(input: &str) -> Option<Annotation> {
    if input.starts_with("//") {
        return Some(Annotation {
            annotation_type: AnnotationType::Comment,
            bytes: 0..input.len(),
        });
    }

    None
}

impl SyntaxHighlighter for RustHighlighter {
    fn highlight(&mut self, _idx: usize, line: &crate::editor::view::line::Line) {
        let mut res = vec![];

        self.highlight(line, &mut res);
        self.highlights.push(res);
    }

    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(line_idx)
    }
}
