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
const QUIT_TIMES: u8 = 2;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    total_lines: usize,
    current_line: usize,
    modified: bool,
    file: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Size {
    width: u16,
    height: u16,
}

impl From<(u16, u16)> for Size {
    fn from(value: (u16, u16)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status: StatusBar,
    message: MessageBar,
    title: String,
    size: Size,
    quit_time: u8,
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
        let size: Size = terminal::size()?.into();

        let mut editor = Editor::default();
        editor.resize(size);

        if editor.view.load(args.path).is_ok() {
            editor
                .message
                .update_message(String::from("HELP: Ctrl-S = save | Ctrl-Q = quit"));
        } else {
            editor
                .message
                .update_message(String::from("ERR: Could not open file"));
        };

        editor.refresh_status();

        Ok(editor)
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            self.refresh_status();
            // self.refresh_message();
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
            if !matches!(event, Command::Save | Command::Quit) {
                self.reset_quit_time();
            }
            match event {
                Command::Move(direction) => self.view.move_point(direction),
                Command::StartOfLine => self.view.move_to_start_of_line(),
                Command::EndOfLine => self.view.move_to_end_of_line(),
                Command::Resize(size) => self.resize(size),
                Command::Insert(c) => self.view.insert_char(c),
                Command::Enter => self.view.insert_newline(),
                Command::Backspace => self.view.delete_backspace(),
                Command::Delete => self.view.delete(),
                Command::Save => self.handle_save(),
                Command::Quit => self.handle_quit(),
            }
        }
    }

    fn refresh_screen(&mut self) {
        if self.size.height == 0 || self.size.width == 0 {
            return;
        }

        let _ = terminal::hide_caret();

        self.message.render(self.size.height.saturating_sub(1));
        if self.size.height > 1 {
            self.status.render(self.size.height.saturating_sub(2));
        }
        if self.size.height > 2 {
            self.view.render(0);
        }
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

    fn resize(&mut self, size: Size) {
        self.size = size;
        self.view.resize(Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        });
        self.message.resize(Size {
            width: size.width,
            height: 1,
        });
        self.status.resize(Size {
            width: size.width,
            height: 1,
        });
    }

    fn handle_save(&mut self) {
        if self.view.save().is_ok() {
            self.message
                .update_message(String::from("File saved successfully."));
        } else {
            self.message
                .update_message(String::from("Error while saving file."));
        }
    }

    fn handle_quit(&mut self) {
        if self.quit_time + 1 >= QUIT_TIMES || !self.view.get_status().modified {
            self.should_quit = true;
        } else {
            self.quit_time += 1;
            self.message.update_message(format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                QUIT_TIMES - self.quit_time
            ));
        }
    }

    fn reset_quit_time(&mut self) {
        self.quit_time = 0;
        self.message.update_message(String::from(""));
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
