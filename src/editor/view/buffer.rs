use std::{io::Write, ops::Deref, path::PathBuf};

use anyhow::Context;

use crate::editor::view::{cursor::Location, line::Line};

#[derive(Default)]
pub struct Buffer {
    pub file: Option<PathBuf>,
    pub dirty: bool,
    lines: Vec<Line>,
}

impl Buffer {
    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }

    pub fn load(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let contents = std::fs::read_to_string(&path).context("read from file")?;
        self.lines = contents.lines().map(Line::from).collect();
        self.file = Some(path);
        Ok(())
    }

    pub fn insert_char(&mut self, c: char, cursor: &super::cursor::Cursor) {
        if cursor.location().line_index == self.len() {
            self.dirty = true;
            self.lines.push(Line::from(c));
        } else if let Some(line) = self.lines.get_mut(cursor.location().line_index) {
            self.dirty = true;
            line.insert_char(c, cursor.location().grapheme_index);
        }
    }

    pub fn delete(&mut self, cursor: &super::cursor::Cursor) {
        if let Some(line) = self.get(cursor.location().line_index) {
            if cursor.location().grapheme_index >= line.grapheme_count()
                && self.len() > cursor.location().line_index.saturating_add(1)
            {
                self.dirty = true;
                let next_line = self
                    .lines
                    .remove(cursor.location().line_index.saturating_add(1));

                self.lines[cursor.location().line_index].append(next_line);
            } else if cursor.location().grapheme_index < line.grapheme_count() {
                self.dirty = true;
                self.lines[cursor.location().line_index].delete(cursor.location().grapheme_index);
            }
        }
    }

    pub fn insert_newline(&mut self, cursor: &super::cursor::Cursor) {
        if cursor.location().line_index == self.len() {
            self.dirty = true;
            self.lines.push(Line::default())
        } else if let Some(line) = self.lines.get_mut(cursor.location().line_index) {
            self.dirty = true;
            let new = line.split(cursor.location().grapheme_index);
            self.lines
                .insert(cursor.location().line_index.saturating_add(1), new);
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(file) = &self.file {
            let mut file = std::fs::File::create(file).context("create file")?;
            for line in self.lines.iter() {
                writeln!(file, "{line}").context("write to file")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn save_as(&mut self, path: &str) -> Result<(), anyhow::Error> {
        let path = PathBuf::from(path);
        let mut file = std::fs::File::create(&path).context("create file")?;
        for line in self.lines.iter() {
            writeln!(file, "{line}").context("write to file")?;
        }
        self.file = Some(path);
        self.dirty = false;
        Ok(())
    }

    pub fn search(&self, query: &str, location: Location) -> Option<Location> {
        for (line_index, line) in self.lines.iter().enumerate().skip(location.line_index) {
            let from_grapheme_index = if line_index == location.line_index {
                location.grapheme_index
            } else {
                0
            };
            if let Some(grapheme_index) = line.search(query, from_grapheme_index) {
                return Some(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }
        None
    }
}

impl Deref for Buffer {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        self.lines.as_ref()
    }
}

#[test]
fn test_search() {
    let buffer = Buffer {
        lines: vec![Line::from("Test: create a new file.")],
        ..Default::default()
    };
    assert_eq!(
        buffer.search(
            "new",
            Location {
                grapheme_index: 0,
                line_index: 0
            }
        ),
        Some(Location {
            grapheme_index: 15,
            line_index: 0
        })
    );
}
