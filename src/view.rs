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

    pub fn render(&mut self) -> anyhow::Result<()> {
        if !self.render {
            return Ok(());
        }
        self.render = false;

        if self.buffer.is_empty() {
            self.render_welcome()
        } else {
            self.render_buffer()
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
    fn render_line(line: u16, text: impl Display) -> anyhow::Result<()> {
        terminal::move_caret(0, line)?;
        terminal::clear_line()?;
        terminal::print(text)
    }

    fn render_buffer(&self) -> anyhow::Result<()> {
        let (_, rows) = self.size();

        for row in 0..rows.saturating_sub(1) {
            if let Some(text) = self.buffer.get(row as usize) {
                Self::render_line(row, text)?;
            } else {
                Self::render_line(row, "~")?;
            }
        }

        Ok(())
    }

    fn render_welcome(&self) -> anyhow::Result<()> {
        let (_, rows) = self.size();

        for row in 0..rows.saturating_sub(1) {
            if row == rows / 3 {
                self.draw_welcome(row)?;
            } else {
                Self::render_line(row, "~")?;
            }
        }

        Ok(())
    }

    fn draw_welcome(&self, row: u16) -> anyhow::Result<()> {
        let message = format!("{NAME} editor -- version {VERSION}");
        let len = message.len();

        let (col, _) = self.size();
        let col = col as usize;

        let start = col.saturating_sub(len) / 2;
        terminal::move_caret(start as u16, row)?;
        terminal::clear_line()?;

        terminal::print(message)
    }
}
