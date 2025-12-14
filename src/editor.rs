use std::io::stdout;

use anyhow::Context;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        Self::init().context("initialize the oxide")?;

        self.repl().context("run the read-eval-print loop")?;

        Self::terminate().context("terminate the exide...")
    }

    fn init() -> anyhow::Result<()> {
        enable_raw_mode().context("enable raw mode in terminal")?;
        Self::clear_screen()
    }

    fn terminate() -> anyhow::Result<()> {
        disable_raw_mode().context("disable raw mode in terminal")
    }

    fn clear_screen() -> anyhow::Result<()> {
        execute!(stdout(), Clear(ClearType::All)).context("clear screen")
    }

    fn repl(&mut self) -> anyhow::Result<()> {
        loop {
            let event = read().context("read input")?;

            self.evalute_event(event).context("evalute input event")?;

            self.refresh_screen().context("refresh screen")?;
            if self.should_quit {
                break Ok(());
            }
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
            Self::clear_screen()?;
        }

        Ok(())
    }
}
