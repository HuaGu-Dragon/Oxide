use std::{
    fmt::Display,
    io::{Write, stdout},
};

use anyhow::Context;
use crossterm::{
    cursor::{self, Hide},
    queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

pub fn init() -> anyhow::Result<()> {
    enable_raw_mode().context("enable raw mode in terminal")?;
    clear_screen()?;
    move_caret(0, 0)
}

pub fn terminate() -> anyhow::Result<()> {
    execute()?;
    disable_raw_mode().context("disable raw mode in terminal")
}

pub fn clear_screen() -> anyhow::Result<()> {
    queue!(stdout(), terminal::Clear(terminal::ClearType::All)).context("clear screen")
}

pub fn clear_line() -> anyhow::Result<()> {
    queue!(stdout(), terminal::Clear(terminal::ClearType::CurrentLine)).context("clear line")
}

/// Get the terminal's size (cols, rows)
pub fn size() -> anyhow::Result<(u16, u16)> {
    terminal::size().context("get the terminal's size")
}

pub fn move_caret(col: u16, row: u16) -> anyhow::Result<()> {
    queue!(stdout(), cursor::MoveTo(col, row))
        .with_context(|| format!("move cursor into ({col}, {row})"))
}

pub fn hide_caret() -> anyhow::Result<()> {
    queue!(stdout(), Hide).context("hide the cursor")
}

pub fn show_caret() -> anyhow::Result<()> {
    queue!(stdout(), cursor::Show).context("show the cursor")
}

pub fn print(text: impl Display) -> anyhow::Result<()> {
    queue!(stdout(), Print(&text)).with_context(|| format!("print `{text}` into terminal"))
}

pub fn execute() -> anyhow::Result<()> {
    stdout().flush().context("flush stdout")
}
