use std::{fmt::Display, path::PathBuf};

use crate::{terminal, view::buffer::Buffer};

mod buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    size: Size,
    render: bool,
}

struct Size {
    width: u16,
    height: u16,
}

impl View {
    pub fn new() -> anyhow::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            buffer: Buffer::new(),
            size: Size { width, height },
            render: true,
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

    pub fn size(&self) -> (u16, u16) {
        (self.size.width, self.size.height)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.size.width = width;
        self.size.height = height;
        self.render = true;
    }

    // TODO: maybe (x, y) is better?
    fn render_line(line: u16, text: impl Display) {
        let result = terminal::print_at(0, line, false, text);
        debug_assert!(result.is_ok());
    }

    fn render_buffer(&self) {
        let (_, rows) = self.size();

        for row in 0..rows.saturating_sub(1) {
            if let Some(text) = self.buffer.get(row as usize) {
                Self::render_line(row, text);
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
