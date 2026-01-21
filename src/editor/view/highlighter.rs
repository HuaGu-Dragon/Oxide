use crate::editor::{
    FileType,
    annotated::annotation::Annotation,
    view::{
        cursor::Location,
        highlighter::{
            rust::RustHighlighter, search::SearchHighlighter, syntax_highlight::SyntaxHighlighter,
        },
        line::Line,
    },
};

mod rust;
mod search;
mod syntax_highlight;

pub struct Highlighter<'a> {
    syntax: Option<Box<dyn SyntaxHighlighter>>,
    search: SearchHighlighter<'a>,
}

fn create_syntax_highlighter(file_ty: FileType) -> Option<Box<dyn SyntaxHighlighter>> {
    match file_ty {
        FileType::Rust => Some(Box::<RustHighlighter>::default()),
        FileType::Text => None,
    }
}

impl<'a> Highlighter<'a> {
    pub fn new(
        match_word: Option<&'a str>,
        selected_match: Option<Location>,
        file_ty: FileType,
    ) -> Self {
        Self {
            syntax: create_syntax_highlighter(file_ty),
            search: SearchHighlighter::new(match_word, selected_match),
        }
    }

    pub fn get_annotations(&self, line_idx: usize) -> Vec<Annotation> {
        let mut annotations = if let Some(search) = self.search.get_annotations(line_idx) {
            search.clone()
        } else {
            vec![]
        };
        if let Some(ref syntax_highlight) = self.syntax {
            annotations.extend(
                syntax_highlight
                    .get_annotations(line_idx)
                    .cloned()
                    .unwrap_or_default(),
            );
        }

        annotations
    }

    pub fn highlight(&mut self, idx: usize, line: &Line) {
        if let Some(ref mut syntax_highlight) = self.syntax {
            syntax_highlight.highlight(idx, line);
        }
        self.search.highlight(idx, line);
    }
}
