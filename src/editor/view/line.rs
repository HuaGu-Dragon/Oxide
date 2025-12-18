use std::ops::{Add, Range};

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

pub struct Line {
    fragments: Vec<TextFragment>,
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
}

impl From<String> for Line {
    fn from(value: String) -> Self {
        Self::from(&value[..])
    }
}

impl From<&str> for Line {
    fn from(value: &str) -> Self {
        Line {
            fragments: value
                .graphemes(true)
                .map(|grapheme| {
                    let unicode_width = grapheme.width();
                    let rendered_width = match unicode_width {
                        0 | 1 => GraphemeWidth::Half,
                        _ => GraphemeWidth::Full,
                    };
                    let replacement = if unicode_width == 0 { Some('·') } else { None };

                    TextFragment {
                        grapheme: grapheme.to_string(),
                        rendered_width,
                        replacement,
                    }
                })
                .collect(),
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
