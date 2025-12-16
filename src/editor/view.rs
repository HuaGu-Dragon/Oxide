use std::{fmt::Display, path::PathBuf};

use crate::{
    editor::{
        command::Direction,
        view::{buffer::Buffer, cursor::Cursor, location::Location},
    },
    terminal,
};

mod buffer;
mod cursor;
mod line;
mod location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    render: bool,
    buffer: Buffer,
    cursor: Cursor,
    offset: Location,
    size: Size,
}

#[derive(Default)]
struct Size {
    width: u16,
    height: u16,
}

impl View {
    pub fn new() -> anyhow::Result<Self> {
        let (cols, rows) = terminal::size()?;
        Ok(Self {
            render: true,
            size: Size {
                width: cols,
                height: rows,
            },
            ..Default::default()
        })
    }

    pub fn load(&mut self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.buffer.load(path);
            self.render = true;
        }
    }

    pub fn render(&mut self) {
        if !self.render {
            return;
        }
        self.render = false;

        if self.buffer.is_empty() {
            self.render_welcome();
        } else {
            self.render_buffer();
        }
    }

    /// (width, height)
    pub fn size(&self) -> (u16, u16) {
        (self.size.width, self.size.height)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.size.width = width;
        self.size.height = height;
        self.render = true;
    }

    fn scroll_location(&mut self) {
        let (x, y) = self.cursor.at().pos();
        let (width, height) = self.size();
        let (x_offset, y_offset) = self.offset.pos_mut();
        let mut offset_change = false;

        if y < *y_offset {
            *y_offset = y;
            offset_change = true;
        } else if y >= y_offset.saturating_add(height as usize) {
            *y_offset = y.saturating_sub(height.saturating_sub(1) as usize);
            offset_change = true;
        }

        if x < *x_offset {
            *x_offset = x;
            offset_change = true;
        } else if x >= x_offset.saturating_add(width as usize) {
            *x_offset = x.saturating_sub(width.saturating_sub(1) as usize);
            offset_change = true;
        }

        self.render = offset_change;
    }

    // TODO: maybe (x, y) is better?
    fn render_line(line: u16, text: impl Display) {
        let result = terminal::print_at(0, line, true, text);
        debug_assert!(result.is_ok());
    }

    pub fn move_point(&mut self, direction: Direction) {
        let (x, y) = self.cursor.at_mut().pos_mut();
        match direction {
            Direction::Up => *y = y.saturating_sub(1),
            Direction::Down => *y = y.saturating_add(1),
            Direction::Left => *x = x.saturating_sub(1),
            Direction::Right => *x = x.saturating_add(1),
        }

        self.scroll_location();
    }

    pub fn cursor_pos(&self) -> (u16, u16) {
        self.cursor.at().subtract(&self.offset)
    }

    fn render_buffer(&self) {
        let (cols, rows) = self.size();
        let (left, top) = self.offset.pos();

        for row in 0..rows.saturating_sub(1) {
            if let Some(text) = self.buffer.get(top.saturating_add(row as usize)) {
                let right = left.saturating_add(cols as usize);
                Self::render_line(row, text.get(left..right));
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
        let col = col as usize;

        let start = col.saturating_sub(len) / 2;
        let result = terminal::print_at(start as u16, row, true, message);

        debug_assert!(result.is_ok())
    }
}
