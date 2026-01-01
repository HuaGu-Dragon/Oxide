use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{
    editor::{
        DocumentStatus, Size,
        event::Direction,
        ui::UiComponent,
        view::{
            buffer::Buffer,
            cursor::{Cursor, Location},
            line::Line,
        },
    },
    terminal::{self, Position},
};

mod buffer;
mod cursor;
pub mod line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct SearchInfo {
    previous_pos: Cursor,
    previous_offset: Position,
    query: Line,
}

#[derive(Default)]
pub struct View {
    render: bool,
    buffer: Buffer,
    cursor: Cursor,
    offset: Position,
    size: Size,
    search_info: Option<SearchInfo>,
}

impl View {
    pub fn has_file(&self) -> bool {
        self.buffer.has_file()
    }

    pub fn load(&mut self, path: Option<PathBuf>) -> anyhow::Result<()> {
        if let Some(path) = path {
            self.buffer.load(path)?;
            self.set_render(true);
        }
        Ok(())
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.len(),
            current_line: self.cursor.location().line_index,
            modified: self.buffer.dirty,
            file: self
                .buffer
                .file
                .as_deref()
                .and_then(Path::file_name)
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("[No Name]")),
        }
    }

    /// (width, height)
    pub fn size(&self) -> (u16, u16) {
        (self.size.width, self.size.height)
    }

    fn scroll_vertically(&mut self, to: usize) {
        let (_, height) = self.size();
        let offset_changed = if to < self.offset.row {
            self.offset.row = to;
            true
        } else if to >= self.offset.row.saturating_add(height as usize) {
            self.offset.row = to.saturating_sub(height as usize).saturating_add(1);
            true
        } else {
            false
        };

        self.set_render(self.render | offset_changed);
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let (width, _) = self.size();
        let offset_changed = if to < self.offset.col {
            self.offset.col = to;
            true
        } else if to >= self.offset.col.saturating_add(width as usize) {
            self.offset.col = to.saturating_sub(width as usize).saturating_add(1);
            true
        } else {
            false
        };

        self.set_render(self.render | offset_changed);
    }

    fn scroll_buffer(&mut self) {
        let Position { col, row } = self.caret_position();

        self.scroll_horizontally(col);
        self.scroll_vertically(row);
    }

    fn center_text_location(&mut self) {}

    // TODO: maybe (x, y) is better?
    fn render_line(line: u16, text: impl Display) {
        let result = terminal::print_at(0, line, true, text);
        debug_assert!(result.is_ok());
    }

    pub fn move_point(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
        }
        self.scroll_buffer();
    }

    fn move_up(&mut self, step: usize) {
        self.cursor.location_mut().line_index =
            self.cursor.location().line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.cursor.location_mut().line_index =
            self.cursor.location().line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self) {
        if self.cursor.location().grapheme_index > 0 {
            self.cursor.location_mut().grapheme_index -= 1;
        } else if self.cursor.location().line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .get(self.cursor.location().line_index)
            .map_or(0, |line| line.grapheme_count());
        if self.cursor.location().grapheme_index < line_width {
            self.cursor.location_mut().grapheme_index += 1;
        } else {
            self.move_down(1);
            self.move_to_start_of_line();
        }
    }

    pub fn move_to_start_of_line(&mut self) {
        self.cursor.location_mut().grapheme_index = 0;
    }

    pub fn move_to_end_of_line(&mut self) {
        self.cursor.location_mut().grapheme_index = self
            .buffer
            .get(self.cursor.location().line_index)
            .map_or(0, |line| line.grapheme_count())
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.cursor.location_mut().grapheme_index = self
            .buffer
            .get(self.cursor.location().line_index)
            .map_or(0, |line| {
                std::cmp::min(line.grapheme_count(), self.cursor.location().grapheme_index)
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.cursor.location_mut().line_index =
            std::cmp::min(self.cursor.location().line_index, self.buffer.len());
    }

    fn render_buffer(&self) {
        let (cols, rows) = self.size();
        let (left, top) = self.offset.pos();

        for row in 0..rows {
            if let Some(text) = self.buffer.get(top.saturating_add(row as usize)) {
                let right = left.saturating_add(cols as usize);
                Self::render_line(row, text.get_visable_graphemes(left..right));
            } else {
                Self::render_line(row, "~");
            }
        }
    }

    fn render_welcome(&self) {
        let (_, rows) = self.size();

        for row in 0..rows.saturating_sub(1) {
            if row == rows / 3 {
                self.draw_welcome(row);
            } else {
                Self::render_line(row, "~");
            }
        }
    }

    fn draw_welcome(&self, row: u16) {
        let message = format!("{NAME} editor -- version {VERSION}");
        let len = message.len();

        let (col, _) = self.size();
        if col == 0 {
            return;
        }
        let col = col as usize;

        let start = col.saturating_sub(len) / 2;
        let result = terminal::print_at(start as u16, row, true, message);

        debug_assert!(result.is_ok())
    }

    fn caret_position(&self) -> Position {
        let cursor = self.cursor.location();
        let row = cursor.line_index;
        let col = self
            .buffer
            .get(row)
            .map_or(0, |line| line.width_until(cursor.grapheme_index));
        Position { col, row }
    }

    pub fn cursor_pos(&self) -> (u16, u16) {
        self.caret_position().subtract(&self.offset)
    }

    pub fn insert_char(&mut self, c: char) {
        let old_len = self
            .buffer
            .get(self.cursor.location().line_index)
            .map_or(0, |line| line.grapheme_count());

        self.buffer.insert_char(c, &self.cursor);

        let new_len = self
            .buffer
            .get(self.cursor.location().line_index)
            .map_or(0, |line| line.grapheme_count());

        let detla = new_len.saturating_sub(old_len);
        if detla > 0 {
            self.move_point(Direction::Right);
        }

        self.set_render(true);
    }

    pub fn delete(&mut self) {
        self.buffer.delete(&self.cursor);
        self.set_render(true);
    }

    pub fn delete_backspace(&mut self) {
        if !matches!(
            self.cursor.location(),
            Location {
                line_index: 0,
                grapheme_index: 0,
            }
        ) {
            self.move_point(Direction::Left);
            self.delete();
        }
    }

    pub fn insert_newline(&mut self) {
        self.buffer.insert_newline(&self.cursor);
        self.move_point(Direction::Right);
        self.set_render(true);
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, path: &str) -> anyhow::Result<()> {
        self.buffer.save_as(path)
    }

    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            previous_pos: self.cursor,
            previous_offset: self.offset,
            query: Line::default(),
        });
    }

    pub fn search(&mut self, query: &str) {
        if let Some(ref mut search_info) = self.search_info {
            search_info.query = Line::from(query);
        }
        self.search_from(self.cursor.location());
    }

    fn search_from(&mut self, from: Location) {
        if let Some(ref search_info) = self.search_info {
            let query = &search_info.query;
            if query.is_empty() {
                return;
            }

            if let Some(location) = self.buffer.search(query, from) {
                self.cursor = Cursor::new(location);
                self.center_text_location();
            }
        }
    }

    pub fn search_next(&mut self) {
        if let Some(ref search_info) = self.search_info {
            let step_right = std::cmp::max(search_info.query.grapheme_count(), 1);
            let location = Location {
                grapheme_index: self.cursor.location().grapheme_index,
                line_index: self.cursor.location().line_index.saturating_add(step_right),
            };

            self.search_from(location);
        }
    }

    pub fn dismiss_search(&mut self) {
        if let Some(ref mut search_info) = self.search_info {
            self.cursor = search_info.previous_pos;
            self.offset = search_info.previous_offset;
            self.set_render(true);
        }
        self.search_info = None;
    }

    pub fn exit_search(&mut self) {
        self.search_info = None
    }
}

impl UiComponent for View {
    fn set_render(&mut self, render: bool) {
        self.render = render;
    }

    fn needs_render(&self) -> bool {
        self.render
    }

    fn set_size(&mut self, width: u16, height: u16) {
        self.size = Size { width, height };
        self.scroll_buffer();
    }

    fn draw(&mut self, _y: u16) -> anyhow::Result<()> {
        if self.size.height == 0 {
            anyhow::bail!("terminal size is zero")
        }

        if self.buffer.is_empty() {
            self.render_welcome();
        } else {
            self.render_buffer();
        }

        Ok(())
    }
}
