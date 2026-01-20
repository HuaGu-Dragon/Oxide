use std::collections::HashMap;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{cursor::Location, line::Line},
};

mod rust;
mod search;
mod syntax_highlight;

pub struct Highlighter<'a> {
    match_word: Option<&'a str>,
    selected_match: Option<Location>,
    highlights: HashMap<usize, Vec<Annotation>>,
}

impl<'a> Highlighter<'a> {
    pub fn new(match_word: Option<&'a str>, selected_match: Option<Location>) -> Self {
        Self {
            match_word,
            selected_match,
            highlights: HashMap::new(),
        }
    }

    pub fn get_annotations(&self, idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }

    pub fn hightlight_digits(line: &Line, res: &mut Vec<Annotation>) {
        line.chars().enumerate().for_each(|(idx, ch)| {
            if ch.is_ascii_digit() {
                res.push(Annotation {
                    annotation_type: AnnotationType::Digit,
                    bytes: idx..idx.saturating_add(1),
                });
            }
        });
    }

    fn highlight_match(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        if let Some(word) = self.match_word {
            line.find_all(word, 0..line.len())
                .iter()
                .for_each(|(start, _)| {
                    res.push(Annotation {
                        annotation_type: AnnotationType::Match,
                        bytes: *start..start.saturating_add(word.len()),
                    });
                });
        }
    }

    fn highlight_selected_match(&mut self, res: &mut Vec<Annotation>) {
        if let Some(selected_match) = self.selected_match
            && let Some(match_word) = self.match_word
            && !match_word.is_empty()
        {
            let start = selected_match.grapheme_index;
            res.push(Annotation {
                annotation_type: AnnotationType::SelectedMatch,
                bytes: start..start.saturating_add(match_word.len()),
            });
        }
    }

    pub fn highlight(&mut self, idx: usize, line: &Line) {
        let mut res = Vec::new();
        Self::hightlight_digits(line, &mut res);

        self.highlight_match(line, &mut res);
        if let Some(selected_match) = self.selected_match
            && selected_match.line_index == idx
        {
            self.highlight_selected_match(&mut res);
        }

        self.highlights.insert(idx, res);
    }
}
