use std::{fmt::Display, panic, path::Path};

use clap::Parser;
use crossterm::event::{Event, read};

use crate::{
    Cli,
    editor::{
        command::CommandBar,
        control::{Control, State},
        event::{Command, Direction},
        message::MessageBar,
        status::StatusBar,
        ui::UiComponent,
        view::View,
    },
    terminal,
};

pub mod annotated;
mod command;
pub mod control;
mod event;
mod message;
mod status;
mod ui;
mod view;

const NAME: &str = env!("CARGO_PKG_NAME");
const QUIT_TIMES: u8 = 2;

#[derive(Default, PartialEq, Eq)]
enum PromptType {
    Search,
    Save,
    #[default]
    None,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum FileType {
    Rust,
    #[default]
    Text,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Rust => write!(f, "Rust"),
            FileType::Text => write!(f, "Text"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    file: String,
    file_ty: FileType,
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.file, self.file_ty)
    }
}

impl From<&Path> for FileInfo {
    fn from(value: &Path) -> Self {
        let file_ty = if value
            .extension()
            .map(|extension| extension.eq("rs"))
            .unwrap_or(false)
        {
            FileType::Rust
        } else {
            FileType::Text
        };
        Self {
            file: value
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("[No Name]")),
            file_ty,
        }
    }
}

impl From<Option<&Path>> for FileInfo {
    fn from(value: Option<&Path>) -> Self {
        value.map(|path| path.into()).unwrap_or(Self {
            file: String::from("[No Name]"),
            file_ty: FileType::Text,
        })
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    total_lines: usize,
    current_line: usize,
    modified: bool,
    file_info: FileInfo,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
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
    command: CommandBar,
    prompt: PromptType,
    title: String,
    size: Size,
    quit_time: u8,
    control: Control,
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
            editor.message.update_message(String::from(
                "HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit",
            ));
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
        if let Ok(command) = self.control.evaluate(event) {
            if let Command::Resize(size) = command {
                self.resize(size);
                return;
            }

            match self.prompt {
                PromptType::Save => self.handle_event_during_save(command),
                PromptType::Search => self.handle_event_during_search(command),
                PromptType::None => self.handle_event_no_prompt(command),
            }
        }
    }

    fn handle_event_no_prompt(&mut self, command: Command) {
        if matches!(command, Command::Quit) {
            self.handle_quit();
            return;
        }
        self.reset_quit_time();

        match command {
            Command::Move(direction) => self.view.move_point(direction),
            Command::Insert(c) => self.view.insert_char(c),
            Command::Delete => self.view.delete(),
            Command::Backspace => self.view.delete_backspace(),
            Command::StartOfLine => self.view.move_to_start_of_line(),
            Command::EndOfLine => self.view.move_to_end_of_line(),
            Command::Enter => self.view.insert_newline(),
            Command::Save => self.handle_save(),
            Command::Search => self.set_prompt(PromptType::Search),
            Command::Dismiss => {}
            Command::Resize(_) | Command::Quit => unreachable!(),
            Command::Switch(_) => {}
            Command::NextWord => self.view.move_to_next_word(),
            Command::PreviousWord => self.view.move_to_previous_word(),
            Command::OpenLineBelow => self.view.open_new_line(),
        }
    }

    fn handle_event_during_search(&mut self, command: Command) {
        match command {
            Command::Insert(_) | Command::Backspace | Command::Delete => {
                self.command.handle_edit_command(command);
                let query = self.command.get_value();
                self.view.search_forward(&query);
            }
            Command::Move(Direction::Up | Direction::Left) => self.view.search_prev(),
            Command::Move(Direction::Down | Direction::Right) => self.view.search_next(),
            Command::Dismiss | Command::Switch(State::Normal) => {
                self.set_prompt(PromptType::None);
                self.view.dismiss_search();
            }
            Command::Enter => {
                self.set_prompt(PromptType::None);
                self.view.exit_search();
            }
            Command::Quit
            | Command::StartOfLine
            | Command::EndOfLine
            | Command::Save
            | Command::Search => {}
            Command::Switch(_) => {}
            Command::Resize(_) => unreachable!(),
            Command::NextWord | Command::PreviousWord | Command::OpenLineBelow => {}
        }
    }

    fn handle_event_during_save(&mut self, command: Command) {
        match command {
            Command::Resize(_) => unreachable!(),
            Command::Move(_)
            | Command::Quit
            | Command::StartOfLine
            | Command::EndOfLine
            | Command::Save
            | Command::Search => {}
            Command::Insert(_) | Command::Backspace | Command::Delete => {
                self.command.handle_edit_command(command)
            }
            Command::Dismiss => {
                self.set_prompt(PromptType::None);
                self.message.update_message("Save aborted.".to_string());
            }
            Command::Enter => {
                let file = self.command.get_value();
                self.save(Some(&file));
                self.set_prompt(PromptType::None);
            }
            // TODO
            Command::Switch(_) => {}
            Command::NextWord | Command::PreviousWord | Command::OpenLineBelow => {}
        }
    }
    fn refresh_screen(&mut self) {
        if self.size.height == 0 || self.size.width == 0 {
            return;
        }

        let _ = terminal::hide_caret();

        if self.in_prompt() {
            self.command.render(self.size.height.saturating_sub(1));
        } else {
            self.message.render(self.size.height.saturating_sub(1));
        }
        if self.size.height > 1 {
            self.status.render(self.size.height.saturating_sub(2));
        }
        if self.size.height > 2 {
            self.view.render(0);
        }
        let (col, row) = if self.in_prompt() {
            (
                self.command.caret_pos_col() as u16,
                self.size.height.saturating_sub(1),
            )
        } else {
            self.view.cursor_pos()
        };

        let _ = terminal::move_caret(col, row);
        let _ = terminal::show_caret();
        let _ = terminal::execute();
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();

        let title = &status.file_info.file;

        if title != &self.title && terminal::set_title(format!("{} - {NAME}", title)).is_ok() {
            self.title = title.to_string()
        }
        self.status.update_status(status);
    }

    fn resize(&mut self, size: Size) {
        self.size = size;
        self.view.resize(Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        });
        let size = Size {
            width: size.width,
            height: 1,
        };
        self.message.resize(size);
        self.status.resize(size);
        self.command.resize(size);
    }

    fn handle_save(&mut self) {
        if self.view.has_file() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn save(&mut self, file: Option<&str>) {
        let res = if let Some(name) = file {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if res.is_ok() {
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

    fn set_prompt(&mut self, prompt: PromptType) {
        match prompt {
            PromptType::Search => {
                self.view.enter_search();
                self.command
                    .set_prompt("Search (Esc to cancel): ".to_string());
            }
            PromptType::Save => self.command.set_prompt("Save as: ".to_string()),
            PromptType::None => self.message.set_render(true),
        }

        self.command.clear();
        self.prompt = prompt;
    }

    fn in_prompt(&self) -> bool {
        self.prompt != PromptType::None
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
