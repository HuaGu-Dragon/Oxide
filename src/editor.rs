use anyhow::Context;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};

pub struct Editor {}

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> anyhow::Result<()> {
        enable_raw_mode().context("enable raw mode in terminal")?;

        loop {
            let event = read().context("read input")?;
            println!("{event:?}");
            if let Event::Key(e) = event
                && let KeyCode::Char('q') = e.code
                && let KeyModifiers::CONTROL = e.modifiers
            {
                break;
            }
        }

        disable_raw_mode().context("disable raw mode in terminal")?;

        Ok(())
    }
}
