use std::{
    fmt::Display,
    ops::{Add, Range},
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum GraphemeWidth {
    Half,
    Full,
}

impl Add<usize> for GraphemeWidth {
    type Output = usize;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            GraphemeWidth::Half => rhs.saturating_add(1),
            GraphemeWidth::Full => rhs.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
    start_byte_idx: usize,
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.string)
    }
}

impl Line {
    pub fn get_visable_graphemes(&self, range: Range<usize>) -> String {
        let mut result = String::new();
        let mut cur_pos = 0;

        for fragment in self.fragments.iter() {
            let fragment_end = fragment.rendered_width + cur_pos;

            if cur_pos >= range.end {
                break;
            }

            if fragment_end > range.start {
                if fragment_end > range.end || cur_pos < range.start {
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }

            cur_pos = fragment_end;
        }

        result
    }

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn width(&self) -> usize {
        self.width_until(self.grapheme_count())
    }

    pub fn clear(&mut self) {
        self.fragments.clear();
        self.string.clear();
    }

    pub fn insert_char(&mut self, c: char, grapheme_index: usize) {
        if let Some(fragment) = self.fragments.get(grapheme_index) {
            self.string.insert(fragment.start_byte_idx, c);
        } else {
            self.string.push(c);
        }
        self.rebuild_fragments();
    }

    pub fn append_char(&mut self, c: char) {
        self.insert_char(c, self.grapheme_count());
    }

    pub fn delete(&mut self, grapheme_index: usize) {
        if let Some(fragment) = self.fragments.get(grapheme_index) {
            let start = fragment.start_byte_idx;
            let end = start.saturating_add(fragment.grapheme.len());

            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn append(&mut self, next_line: Line) {
        self.string.push_str(&next_line.string);
        self.rebuild_fragments();
    }

    pub fn split(&mut self, grapheme_index: usize) -> Self {
        if let Some(grapheme) = self.fragments.get(grapheme_index) {
            let remainder = self.string.split_off(grapheme.start_byte_idx);
            self.rebuild_fragments();
            Self::from(remainder)
        } else {
            Self::default()
        }
    }

    fn rebuild_fragments(&mut self) {
        self.fragments = str_to_fragments(&self.string);
    }
}

fn replace_charactor(grapheme: &str) -> Option<char> {
    let width = grapheme.width();
    match grapheme {
        " " => None,
        "\t" => Some(' '),
        _ if width > 0 && grapheme.trim().is_empty() => Some('␣'),
        _ if width == 0 => {
            let mut chars = grapheme.chars();
            if let Some(ch) = chars.next()
                && ch.is_control()
                && chars.next().is_none()
            {
                return Some('▯');
            }
            Some('·')
        }
        _ => None,
    }
}

fn str_to_fragments(value: &str) -> Vec<TextFragment> {
    value
        .grapheme_indices(true)
        .map(|(index, grapheme)| {
            let (rendered_width, replacement) = replace_charactor(grapheme).map_or_else(
                || {
                    let unicode_width = grapheme.width();
                    let rendered_width = match unicode_width {
                        0 | 1 => GraphemeWidth::Half,
                        _ => GraphemeWidth::Full,
                    };
                    (rendered_width, None)
                },
                |replacement| (GraphemeWidth::Half, Some(replacement)),
            );

            TextFragment {
                grapheme: grapheme.to_string(),
                rendered_width,
                replacement,
                start_byte_idx: index,
            }
        })
        .collect()
}

impl From<String> for Line {
    fn from(value: String) -> Self {
        Self::from(&value[..])
    }
}

impl From<char> for Line {
    fn from(value: char) -> Self {
        Self::from(value.to_string())
    }
}

impl From<&str> for Line {
    fn from(value: &str) -> Self {
        Self {
            fragments: str_to_fragments(value),
            string: value.to_string(),
        }
    }
}

#[test]
fn test_line_range() {
    let line = Line::from("Hello, world!");
    assert_eq!(line.grapheme_count(), 13);
    assert_eq!(&line.get_visable_graphemes(0..5), "Hello");
    assert_eq!(&line.get_visable_graphemes(7..13), "world!");
    assert_eq!(&line.get_visable_graphemes(13..18), "");
}

#[test]
fn test_unicode() {
    let line = Line::from("𝒻𝒶𝓃𝒸𝓎!");
    assert_eq!(line.grapheme_count(), 6);
    assert_eq!(&line.get_visable_graphemes(0..6), "𝒻𝒶𝓃𝒸𝓎!");
    assert_eq!(&line.get_visable_graphemes(7..100), "");
}

#[test]
fn test_width_charactor() {
    let line = Line::from("Ａ");
    assert_eq!(line.grapheme_count(), 1);
    assert_eq!(&line.get_visable_graphemes(0..1), "⋯");
    assert_eq!(&line.get_visable_graphemes(0..2), "Ａ");
    assert_eq!(&line.get_visable_graphemes(2..3), "");
}

#[test]
fn esc_and_bell() {
    let line = Line::from(" ");

    assert_eq!(line.grapheme_count(), 3);
    assert_eq!(line.fragments[0].grapheme.width(), 1);
    assert_eq!(line.fragments[2].grapheme.width(), 1);
}
