use std::{
    fmt::Display,
    ops::{Add, Deref, Range, Sub},
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::editor::annotated::{AnnotatedString, annotation::AnnotationType};

#[derive(Debug, Clone, Copy)]
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

impl Sub<GraphemeWidth> for usize {
    type Output = usize;

    fn sub(self, rhs: GraphemeWidth) -> Self::Output {
        match rhs {
            GraphemeWidth::Half => self.saturating_sub(1),
            GraphemeWidth::Full => self.saturating_sub(2),
        }
    }
}

#[derive(Debug, Clone)]
struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
    start_byte_idx: usize,
}

#[derive(Default, Debug, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl Line {
    pub fn get_visable_graphemes(&self, range: Range<usize>) -> String {
        self.get_annotated_visiable_string(range, None, None)
            .to_string()
    }

    pub fn get_annotated_visiable_string(
        &self,
        range: Range<usize>,
        query: Option<&str>,
        select_match: Option<usize>,
    ) -> AnnotatedString {
        let mut res = AnnotatedString::from(&self.string[..]);

        if let Some(query) = query
            && !query.is_empty()
        {
            self.find_all(query, 0..self.string.len()).iter().for_each(
                |(start_idx, grapheme_idx)| {
                    if let Some(select_match) = select_match
                        && *grapheme_idx == select_match
                    {
                        res.add_annotation(
                            AnnotationType::SelectedMatch,
                            *start_idx..start_idx.saturating_add(query.len()),
                        );
                        return;
                    }
                    res.add_annotation(
                        AnnotationType::Match,
                        *start_idx..start_idx.saturating_add(query.len()),
                    );
                },
            );
        }

        let mut fragment_start = self.width();
        for fragnment in self.fragments.iter().rev() {
            let fragment_end = fragment_start;
            fragment_start = fragment_start - fragnment.rendered_width;

            if fragment_start > range.end {
                continue;
            }

            if fragment_start < range.end && fragment_end > range.end {
                res.replace(fragnment.start_byte_idx..self.string.len(), "⋯");
                continue;
            } else if fragment_start == range.end {
                res.replace(fragnment.start_byte_idx..self.string.len(), "");
                continue;
            }

            if fragment_end <= range.start {
                res.replace(
                    0..fragnment
                        .start_byte_idx
                        .saturating_add(fragnment.grapheme.len()),
                    "",
                );
                break;
            } else if fragment_start < range.start && fragment_end > range.start {
                res.replace(
                    0..fragnment
                        .start_byte_idx
                        .saturating_add(fragnment.grapheme.len()),
                    "⋯",
                );
                break;
            }

            if fragment_start >= range.start
                && fragment_end <= range.end
                && let Some(replacement) = fragnment.replacement
            {
                let start_byte_idx = fragnment.start_byte_idx;
                let end_byte_idx = start_byte_idx.saturating_add(fragnment.grapheme.len());
                res.replace(start_byte_idx..end_byte_idx, &replacement.to_string());
            }
        }

        res
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

    fn byte_idx_to_grapheme_index(&self, byte_index: usize) -> Option<usize> {
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_index)
    }

    fn grapheme_index_to_byte_idx(&self, grapheme_index: usize) -> Option<usize> {
        self.fragments
            .get(grapheme_index)
            .map(|fragment| fragment.start_byte_idx)
    }

    pub fn search_forward(&self, query: &str, grapheme_index: usize) -> Option<usize> {
        let byte_index = if grapheme_index == self.grapheme_count() {
            None
        } else {
            self.grapheme_index_to_byte_idx(grapheme_index)
        }?;

        self.string
            .get(byte_index..)
            .and_then(|s| s.find(query))
            .and_then(|index| self.byte_idx_to_grapheme_index(index.saturating_add(byte_index)))
    }

    pub fn search_backward(&self, query: &str, grapheme_index: usize) -> Option<usize> {
        let byte_index = if grapheme_index == self.grapheme_count() {
            Some(self.string.len())
        } else {
            self.grapheme_index_to_byte_idx(grapheme_index)
        }?;

        self.string
            .get(..byte_index)
            .and_then(|s| s.rfind(query))
            .and_then(|index| self.byte_idx_to_grapheme_index(index))
    }

    /// return: (bytes_index, grapheme_index)
    fn find_all(&self, query: &str, range: Range<usize>) -> Vec<(usize, usize)> {
        self.string
            .get(range.start..range.end)
            .map_or_else(Vec::new, |substr| {
                substr
                    .match_indices(query)
                    .filter_map(|(relative_idx, _)| {
                        let abs_idx = range.start.saturating_add(relative_idx);
                        self.byte_idx_to_grapheme_index(abs_idx)
                            .map(|grapheme_idx| (abs_idx, grapheme_idx))
                    })
                    .collect()
            })
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

impl Deref for Line {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
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

#[test]
fn test_search() {
    let line = Line::from("hello world");
    assert_eq!(line.search_forward("hello", 0), Some(0));
    assert_eq!(line.search_forward("world", 0), Some(6));

    let line = Line::from("你好 世界 ");

    assert_eq!(line.search_forward("你好", 0), Some(0));
    assert_eq!(line.search_forward("世界", 0), Some(3));

    assert_eq!(line.search_backward("你好", 5), Some(0));
    assert_eq!(line.search_backward("世界", 5), Some(3));
}

#[test]
fn annotation() {
    let line = Line::from("Control");
    let annotation = line.get_annotated_visiable_string(0..7, Some("o"), Some(2));
    let mut iter = annotation.into_iter();

    assert!(iter.next().is_some()); // C
    assert!(iter.next().is_some()); // o
    assert!(iter.next().is_some()); // ntr
    assert!(iter.next().is_some()); // o
    assert!(iter.next().is_some()); // l
    assert!(iter.next().is_none());
}
