use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};

use crate::terminal;

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
                ..
            })
        ) {
            self.should_quit = true;
        }

        Ok(())
    }

    fn refresh_screen(&mut self) -> anyhow::Result<()> {
        if self.should_quit {
            terminal::clear_screen()?;
        } else {
            Self::draw_rows()?;
            terminal::move_cursor(0, 0)?;
        }

        Ok(())
    }

    fn draw_rows() -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;

        for row in 0..rows.saturating_sub(1) {
            terminal::move_cursor(0, row)?;

            terminal::print("~")?;
        }

        Ok(())
    }
}
