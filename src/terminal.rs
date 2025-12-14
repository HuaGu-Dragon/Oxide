use std::{fmt::Display, io::stdout};

use anyhow::Context;
use crossterm::{
    cursor, execute,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

pub fn init() -> anyhow::Result<()> {
    enable_raw_mode().context("enable raw mode in terminal")?;
    clear_screen()?;
    move_cursor(0, 0)
}

pub fn terminate() -> anyhow::Result<()> {
    disable_raw_mode().context("disable raw mode in terminal")
}

pub fn clear_screen() -> anyhow::Result<()> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).context("clear screen")
}

/// Get the terminal's size (cols, rows)
pub fn size() -> anyhow::Result<(u16, u16)> {
    terminal::size().context("get the terminal's size")
}

pub fn move_cursor(col: u16, row: u16) -> anyhow::Result<()> {
    execute!(stdout(), cursor::MoveTo(col, row))
        .with_context(|| format!("move cursor into ({col}, {row})"))
}

pub fn print(text: impl Display) -> anyhow::Result<()> {
    execute!(stdout(), Print(&text)).with_context(|| format!("print `{text}` into terminal"))
}
