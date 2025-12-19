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
        if cursor.location().line_index == self.len() {
            self.lines.push(Line::from(c));
        } else if let Some(line) = self.lines.get_mut(cursor.location().line_index) {
            line.insert_char(c, cursor.location().grapheme_index);
        }
    }

    pub fn delete(&mut self, cursor: &super::cursor::Cursor) {
        if let Some(line) = self.get(cursor.location().line_index) {
            if cursor.location().grapheme_index >= line.grapheme_count()
                && self.len() > cursor.location().line_index.saturating_add(1)
            {
                let next_line = self
                    .lines
                    .remove(cursor.location().line_index.saturating_add(1));

                self.lines[cursor.location().line_index].append(next_line);
            } else if cursor.location().grapheme_index < line.grapheme_count() {
                self.lines[cursor.location().line_index].delete(cursor.location().grapheme_index);
            }
        }
    }

    pub fn insert_newline(&mut self, cursor: &super::cursor::Cursor) {
        if cursor.location().line_index == self.len() {
            self.lines.push(Line::default());
        } else if let Some(line) = self.lines.get_mut(cursor.location().line_index) {
            let new = line.split(cursor.location().grapheme_index);
            self.lines
                .insert(cursor.location().line_index.saturating_add(1), new);
        }
    }
}

impl Deref for Buffer {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        self.lines.as_ref()
    }
}
