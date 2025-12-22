use std::panic;

use clap::Parser;
use crossterm::event::{Event, read};

use crate::{
    Cli,
    editor::{
        command::Command, message::MessageBar, status::StatusBar, ui::UiComponent, view::View,
    },
    terminal,
};

mod command;
mod message;
mod status;
mod ui;
mod view;

const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    total_lines: usize,
    current_line: usize,
    modified: bool,
    file: String,
}

pub struct Editor {
    should_quit: bool,
    view: View,
    status: StatusBar,
    message: MessageBar,
    title: String,
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

        let mut editor = Self {
            should_quit: false,
            view: View::new(2)?,
            status: StatusBar::new(1),
            title: String::new(),
            message: Default::default(),
        };

        editor.view.load(args.path);

        editor.refresh_status();
        editor.refresh_message();

        Ok(editor)
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            self.refresh_status();
            self.refresh_message();
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
                    self.status.resize(width as usize, height as usize);
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
        self.message.render();

        let (col, row) = self.view.cursor_pos();

        let _ = terminal::move_caret(col, row);
        let _ = terminal::show_caret();
        let _ = terminal::execute();
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();

        let title = status.file;

        if title != self.title && terminal::set_title(format!("{} - {NAME}", title)).is_ok() {
            self.title = title.to_string()
        }
    }

    fn refresh_message(&mut self) {
        self.message
            .update_message(String::from("HELP: Ctrl-S = save | Ctrl-Q = quit"));
    }
}

impl DocumentStatus {
    pub fn modified_indicator(&self) -> &'static str {
        if self.modified { "(modified)" } else { "" }
    }

    pub fn line_count(&self) -> String {
        format!("{} lines", self.total_lines)
    }

    pub fn position_indicator(&self) -> String {
        format!(
            "{}/{}",
            self.current_line.saturating_add(1),
            self.total_lines
        )
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
