use std::collections::HashMap;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{cursor::Location, line::Line},
};

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

    pub fn highlight(&mut self, idx: usize, line: &Line) {
        let mut res = Vec::new();
        Self::hightlight_digits(line, &mut res);

        if let Some(selected_match) = self.selected_match
            && selected_match.line_index == idx
        {
            // TODO: highlight selected match
        }

        self.highlights.insert(idx, res);
    }
}
