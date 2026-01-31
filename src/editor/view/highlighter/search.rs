use std::collections::HashMap;

use crate::editor::{
    annotated::annotation::{Annotation, AnnotationType},
    view::{cursor::Location, highlighter::syntax_highlight::SyntaxHighlighter, line::Line},
};

pub struct SearchHighlighter<'a> {
    match_word: Option<&'a str>,
    selected_match: Option<Location>,
    highlights: HashMap<usize, Vec<Annotation>>,
}

impl<'a> SearchHighlighter<'a> {
    pub fn new(match_word: Option<&'a str>, selected_match: Option<Location>) -> Self {
        Self {
            match_word,
            selected_match,
            highlights: HashMap::new(),
        }
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

    fn highlight_selected_match(&mut self, line: &Line, res: &mut Vec<Annotation>) {
        if let Some(selected_match) = self.selected_match
            && let Some(match_word) = self.match_word
            && !match_word.is_empty()
            && let Some(start) = line.grapheme_index_to_byte_idx(selected_match.grapheme_index)
        {
            res.push(Annotation {
                annotation_type: AnnotationType::SelectedMatch,
                bytes: start..start.saturating_add(match_word.len()),
            });
        }
    }
}

impl<'a> SyntaxHighlighter for SearchHighlighter<'a> {
    fn highlight(&mut self, idx: usize, line: &Line) {
        let mut res = Vec::new();

        self.highlight_match(line, &mut res);
        if let Some(selected_match) = self.selected_match
            && selected_match.line_index == idx
        {
            self.highlight_selected_match(line, &mut res);
        }

        self.highlights.insert(idx, res);
    }

    fn get_annotations(&self, line_idx: usize) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
