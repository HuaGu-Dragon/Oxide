use std::{ops::Deref, path::PathBuf};

use crate::editor::view::line::Line;

#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,
}

impl Buffer {
    pub fn load(&mut self, path: PathBuf) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            self.lines = contents.lines().map(Line::from).collect();
        }
    }

    pub fn insert_char(&mut self, c: char, cursor: &super::cursor::Cursor) {
        if cursor.location().line_index > self.lines.len() {
            return;
        }

        if cursor.location().line_index == self.lines.len() {
            self.lines.push(Line::from(c));
        } else if let Some(line) = self.lines.get_mut(cursor.location().line_index) {
            line.insert_char(c, cursor.location().grapheme_index);
        }
    }
}

impl Deref for Buffer {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        self.lines.as_ref()
    }
}
