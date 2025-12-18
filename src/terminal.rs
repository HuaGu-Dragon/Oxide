use std::{
    fmt::Display,
    io::{Write, stdout},
};

use anyhow::Context;
use crossterm::{
    cursor::{self, Hide},
    queue,
    style::Print,
    terminal,
};

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub fn subtract(&self, offset: &Position) -> (u16, u16) {
        let x = self.col.saturating_sub(offset.col) as u16;
        let y = self.row.saturating_sub(offset.row) as u16;
        (x, y)
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.col, self.row)
    }
}

pub fn init() -> anyhow::Result<()> {
    terminal::enable_raw_mode().context("enable raw mode in terminal")?;

    enter_alternate()?;

    clear_screen()?;
    execute()
}

pub fn terminate() -> anyhow::Result<()> {
    exit_alternate()?;
    show_caret()?;

    execute()?;
    terminal::disable_raw_mode().context("disable raw mode in terminal")
}

fn enter_alternate() -> anyhow::Result<()> {
    queue!(stdout(), terminal::EnterAlternateScreen).context("enter alternate screen")
}

fn exit_alternate() -> anyhow::Result<()> {
    queue!(stdout(), terminal::LeaveAlternateScreen).context("exit alternate screen")
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

pub fn print_at(col: u16, row: u16, clear: bool, text: impl Display) -> anyhow::Result<()> {
    move_caret(col, row)?;
    if clear {
        clear_line()?;
    }

    print(text)
}

pub fn execute() -> anyhow::Result<()> {
    stdout().flush().context("flush stdout")
}
