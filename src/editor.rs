use std::{panic, path::PathBuf};

use clap::Parser;
use crossterm::event::{Event, read};

use crate::{
    Cli,
    editor::{command::Command, status::StatusBar, view::View},
    terminal,
};

mod command;
mod status;
mod view;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    total_lines: usize,
    current_line: usize,
    modified: bool,
    file: Option<PathBuf>,
}

pub struct Editor {
    should_quit: bool,
    /// The current position of the caret in the editor.
    view: View,
    status: StatusBar,
}

impl Editor {
    pub fn new() -> anyhow::Result<Self> {
        let current_hook = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            let _ = terminal::terminate();
            current_hook(panic_info);
        }));
        terminal::init()?;
        let args = Cli::parse();

        let mut view = View::new(2)?;
        view.load(args.path);

        Ok(Self {
            should_quit: false,
            view,
            status: StatusBar::new(1),
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => self.evalute_event(event),
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {_err:?}")
                    }
                }
            }
            let status = self.view.get_status();
            self.status.update_status(status);
        }
    }

    fn evalute_event(&mut self, event: Event) {
        if let Ok(event) = Command::try_from(event) {
            match event {
                Command::Move(direction) => self.view.move_point(direction),
                Command::Resize(width, height) => {
                    self.view.resize(width, height);
                    self.status.resize(width, height);
                }
                Command::Insert(c) => self.view.insert_char(c),
                Command::Enter => self.view.insert_newline(),
                Command::Backspace => self.view.delete_backspace(),
                Command::Delete => self.view.delete(),
                Command::Save => self.view.save(),
                Command::Quit => self.should_quit = true,
            }
        }
    }

    fn refresh_screen(&mut self) {
        let _ = terminal::hide_caret();

        self.view.render();
        self.status.render();

        let (col, row) = self.view.cursor_pos();

        let _ = terminal::move_caret(col, row);
        let _ = terminal::show_caret();
        let _ = terminal::execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = terminal::terminate();
        if self.should_quit {
            println!("Goodbye");
        }
    }
}
