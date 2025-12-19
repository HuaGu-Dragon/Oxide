use std::panic;

use clap::Parser;
use crossterm::event::{Event, read};

use crate::{
    Cli,
    editor::{command::Command, view::View},
    terminal,
};

mod command;
mod view;

pub struct Editor {
    should_quit: bool,
    /// The current position of the caret in the editor.
    view: View,
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

        let mut view = View::new()?;
        view.load(args.path);

        Ok(Self {
            should_quit: false,
            view,
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
        }
    }

    fn evalute_event(&mut self, event: Event) {
        if let Ok(event) = Command::try_from(event) {
            match event {
                Command::Move(direction) => self.view.move_point(direction),
                Command::Resize(width, height) => self.view.resize(width, height),
                Command::Insert(c) => self.view.insert_char(c),
                Command::Enter => self.view.insert_newline(),
                Command::Backspace => self.view.delete_backspace(),
                Command::Delete => self.view.delete(),
                Command::Quit => self.should_quit = true,
            }
        }
    }

    fn refresh_screen(&mut self) {
        let _ = terminal::hide_caret();

        self.view.render();

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
