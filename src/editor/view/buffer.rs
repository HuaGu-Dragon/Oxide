use std::{
    io::Write,
    ops::{Deref, Range},
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::editor::{
    annotated::AnnotatedString,
    view::{cursor::Location, highlighter::Highlighter, line::Line},
};

#[derive(Default)]
pub struct Buffer {
    file: Option<PathBuf>,
    dirty: bool,
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

    pub fn search_forward(&self, query: &str, location: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }
        let mut is_first = true;
        for (line_index, line) in self
            .lines
            .iter()
            .enumerate()
            .cycle()
            .skip(location.line_index)
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_index = if is_first {
                is_first = false;
                location.grapheme_index
            } else {
                0
            };
            if let Some(grapheme_index) = line.search_forward(query, from_grapheme_index) {
                return Some(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }
        None
    }

    pub fn search_backward(&self, query: &str, location: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }
        let mut is_first = true;
        for (line_index, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(location.line_index)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_index = if is_first {
                is_first = false;
                location.grapheme_index
            } else {
                line.grapheme_count()
            };
            if let Some(grapheme_index) = line.search_backward(query, from_grapheme_index) {
                return Some(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }
        None
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn get_highlight_substring(
        &self,
        line_idx: usize,
        range: Range<usize>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_idx).map(|line| {
            line.get_annotated_visiable_string(range, highlighter.get_annotations(line_idx))
        })
    }

    pub fn highlight(&self, line_idx: usize, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(line_idx) {
            highlighter.highlight(line_idx, line);
        }
    }
}

impl Deref for Buffer {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        self.lines.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::editor::view::{buffer::Buffer, cursor::Location, line::Line};

    impl Buffer {
        pub fn new(lines: Vec<Line>) -> Self {
            Self {
                lines,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_search() {
        let buffer = Buffer {
            lines: vec![Line::from("Test: create a new file.")],
            ..Default::default()
        };
        assert_eq!(
            buffer.search_forward(
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

        assert_eq!(
            buffer.search_backward(
                "new",
                Location {
                    grapheme_index: 23,
                    line_index: 0
                }
            ),
            Some(Location {
                grapheme_index: 15,
                line_index: 0
            })
        )
    }

    #[test]
    fn search_same() {
        let buffer = Buffer {
            lines: vec![Line::from("new new new new")],
            ..Default::default()
        };
        assert_eq!(
            buffer.search_forward(
                "new",
                Location {
                    grapheme_index: 0,
                    line_index: 0
                }
            ),
            Some(Location {
                grapheme_index: 0,
                line_index: 0
            })
        );

        assert_eq!(
            buffer.search_backward(
                "new",
                Location {
                    grapheme_index: 0,
                    line_index: 0
                }
            ),
            Some(Location {
                grapheme_index: 12,
                line_index: 0
            })
        )
    }
}
