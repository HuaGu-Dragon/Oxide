use std::path::PathBuf;

use crate::{terminal, view::buffer::Buffer};

mod buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
        }
    }

    pub fn load(&mut self, path: PathBuf) {
        self.buffer.load(path);
    }

    pub fn render(&self) -> anyhow::Result<()> {
        if self.buffer.is_empty() {
            self.render_welcome()
        } else {
            self.render_buffer()
        }
    }

    fn render_buffer(&self) -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;

        for row in 0..rows.saturating_sub(1) {
            terminal::move_caret(0, row)?;

            terminal::clear_line()?;

            if let Some(line) = self.buffer.get(row as usize) {
                terminal::print(line)?;
            } else {
                terminal::print("~")?;
            }
        }

        Ok(())
    }

    fn render_welcome(&self) -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;

        for row in 0..rows.saturating_sub(1) {
            terminal::move_caret(0, row)?;

            terminal::clear_line()?;

            if row == rows / 3 {
                Self::draw_welcome(row)?;
            } else {
                terminal::print("~")?;
            }
        }

        Ok(())
    }

    fn draw_welcome(row: u16) -> anyhow::Result<()> {
        let message = format!("{NAME} editor -- version {VERSION}");
        let len = message.len();

        let (col, _) = terminal::size()?;
        let col = col as usize;

        let start = col.saturating_sub(len) / 2;
        terminal::move_caret(start as u16, row)?;
        terminal::print(message)?;

        Ok(())
    }
}
