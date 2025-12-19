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
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    bytes: usize,
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for fragment in self.fragments.iter() {
            write!(f, "{}", fragment.grapheme)?;
        }

        Ok(())
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

    pub fn insert_char(&mut self, c: char, grapheme_index: usize) {
        let mut res = String::with_capacity(self.bytes + c.len_utf8());

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                res.push(c);
            }

            res.push_str(&fragment.grapheme);
        }

        if grapheme_index >= self.fragments.len() {
            res.push(c);
        }

        self.bytes += c.len_utf8();
        self.fragments = str_to_fragments(&res);
    }

    pub fn delete(&mut self, grapheme_index: usize) {
        let mut res = String::with_capacity(self.bytes);

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                continue;
            }

            res.push_str(&fragment.grapheme);
        }

        if let Some(value) = self.fragments.get(grapheme_index) {
            self.bytes -= value.grapheme.len();
        }
        self.fragments = str_to_fragments(&res);
    }

    pub fn append(&mut self, next_line: Line) {
        let mut concat = self.to_string();
        concat.push_str(&next_line.to_string());
        self.bytes += next_line.bytes;
        self.fragments = str_to_fragments(&concat);
    }

    pub fn split(&mut self, grapheme_index: usize) -> Self {
        if grapheme_index > self.fragments.len() {
            return Default::default();
        }

        let remainder = self.fragments.split_off(grapheme_index);
        let bytes = remainder
            .iter()
            .map(|fragment| fragment.grapheme.len())
            .sum();

        self.bytes -= bytes;
        Self {
            fragments: remainder,
            bytes,
        }
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
        .graphemes(true)
        .map(|grapheme| {
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
            bytes: value.len(),
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
