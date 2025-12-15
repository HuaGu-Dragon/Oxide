use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};

use crate::terminal;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    /// The current position of the caret in the editor.
    pos: Position,
}

// TODO: move it into the terminal, because it's not editor specific
// editor need the (col, row) of the current buffer not this.
struct Position {
    x: u16,
    y: u16,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            pos: Position { x: 0, y: 0 },
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        terminal::init()?;

        self.repl().context("run the read-eval-print loop")?;

        terminal::terminate()
    }

    fn repl(&mut self) -> anyhow::Result<()> {
        loop {
            self.refresh_screen().context("refresh screen")?;
            if self.should_quit {
                break Ok(());
            }

            let event = read().context("read input")?;

            self.evalute_event(event).context("evalute input event")?;
        }
    }

    fn evalute_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(key_event) => match key_event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    self.should_quit = true;
                }
                KeyEvent {
                    code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                    kind: KeyEventKind::Press,
                    ..
                } => self.move_point(key_event.code)?,
                _ => {}
            },
            Event::Mouse(_mouse_event) => {}
            Event::Paste(_) => {}
            // TODO: cache size of the terminal, use this event to update the size of the terminal
            Event::Resize(_, _) => {}
        }

        Ok(())
    }

    fn move_point(&mut self, code: KeyCode) -> anyhow::Result<()> {
        let pos = &mut self.pos;
        let (cols, rows) = terminal::size()?;

        // TODO: A Cursor struct to handle cursor position and size, like move saturating_left()
        match code {
            KeyCode::Up => {
                pos.y = pos.y.saturating_sub(1);
            }
            KeyCode::Down => {
                pos.y = pos.y.saturating_add(1).min(rows.saturating_sub(1));
            }
            KeyCode::Left => {
                pos.x = pos.x.saturating_sub(1);
            }
            KeyCode::Right => {
                pos.x = pos.x.saturating_add(1).min(cols.saturating_sub(1));
            }
            _ => {}
        }

        Ok(())
    }

    fn refresh_screen(&mut self) -> anyhow::Result<()> {
        terminal::hide_caret()?;
        terminal::move_caret(0, 0)?;

        if self.should_quit {
            terminal::clear_screen()?;
        } else {
            Self::draw_rows()?;
            terminal::move_caret(self.pos.x, self.pos.y)?;
        }

        terminal::show_caret()?;
        terminal::execute()
    }

    fn draw_rows() -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;

        for row in 0..rows.saturating_sub(1) {
            terminal::move_caret(0, row)?;

            terminal::clear_line()?;
            terminal::print("~")?;
            if row == rows / 3 {
                Self::draw_welcome(row)?;
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
