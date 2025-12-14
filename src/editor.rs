use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};

use crate::terminal;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self { should_quit: false }
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
        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            })
        ) {
            self.should_quit = true;
        }

        Ok(())
    }

    fn refresh_screen(&mut self) -> anyhow::Result<()> {
        terminal::hide_cursor()?;

        if self.should_quit {
            terminal::clear_screen()?;
        } else {
            Self::draw_rows()?;
            terminal::move_cursor(0, 0)?;
        }

        terminal::show_cursor()?;
        terminal::execute()
    }

    fn draw_rows() -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;

        for row in 0..rows.saturating_sub(1) {
            terminal::move_cursor(0, row)?;

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
        terminal::move_cursor(start as u16, row)?;
        terminal::print(message)?;

        Ok(())
    }
}
