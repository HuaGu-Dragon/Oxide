use std::{
    fmt::Display,
    io::{Write, stdout},
};

use anyhow::Context;
use crossterm::{
    cursor, queue,
    style::{self, Print},
    terminal,
};

mod attribute;

#[derive(Debug, Clone, Copy, Default)]
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
    disable_line_wrap()?;
    set_cursor_style(cursor::SetCursorStyle::SteadyBlock)?;

    clear_screen()?;
    execute()
}

pub fn terminate() -> anyhow::Result<()> {
    set_cursor_style(cursor::SetCursorStyle::DefaultUserShape)?;
    exit_alternate()?;
    enable_line_wrap()?;
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

fn enable_line_wrap() -> anyhow::Result<()> {
    queue!(stdout(), terminal::EnableLineWrap).context("enable line wrap")
}

fn disable_line_wrap() -> anyhow::Result<()> {
    queue!(stdout(), terminal::DisableLineWrap).context("disable line wrap")
}

pub fn set_cursor_style(style: cursor::SetCursorStyle) -> anyhow::Result<()> {
    queue!(stdout(), style).context("set cursor style")
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
    queue!(stdout(), cursor::Hide).context("hide the cursor")
}

pub fn show_caret() -> anyhow::Result<()> {
    queue!(stdout(), cursor::Show).context("show the cursor")
}

pub fn print(text: impl Display) -> anyhow::Result<()> {
    queue!(stdout(), Print(&text)).with_context(|| format!("print `{text}` into terminal"))
}

pub fn print_inverted(text: impl Display) -> anyhow::Result<()> {
    print(format!(
        "{}{}{}",
        style::Attribute::Reverse,
        text,
        style::Attribute::Reset
    ))
}

pub fn print_at(col: u16, row: u16, clear: bool, text: impl Display) -> anyhow::Result<()> {
    move_caret(col, row)?;
    if clear {
        clear_line()?;
    }

    print(text)
}
pub fn print_inverted_at(
    col: u16,
    row: u16,
    clear: bool,
    text: impl Display,
) -> anyhow::Result<()> {
    move_caret(col, row)?;
    if clear {
        clear_line()?;
    }

    print_inverted(text)
}

pub fn execute() -> anyhow::Result<()> {
    stdout().flush().context("flush stdout")
}

pub fn set_title(title: impl Display) -> anyhow::Result<()> {
    queue!(stdout(), terminal::SetTitle(title)).context("set title")?;
    Ok(())
}
