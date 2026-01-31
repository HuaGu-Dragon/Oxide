use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::editor::event::{Command, Direction};

#[derive(Debug, Default)]
pub struct Control {
    mode: State,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    #[default]
    Normal,
    Insert,
}

impl Control {
    pub fn evaluate(&mut self, event: Event) -> anyhow::Result<Command> {
        if let Event::Key(e) = event {
            return match self.mode {
                State::Normal => self.normal_command(e),
                State::Insert => self.insert_command(e),
            };
        }

        Command::try_from(event)
    }

    fn normal_command(&mut self, event: KeyEvent) -> anyhow::Result<Command> {
        match event {
            KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            } => match (code, modifiers) {
                (KeyCode::Char('i'), KeyModifiers::NONE) => {
                    self.mode = State::Insert;
                    Ok(Command::Switch(State::Insert))
                }
                (KeyCode::Char('i') | KeyCode::Char('I'), KeyModifiers::SHIFT) => {
                    self.mode = State::Insert;
                    Ok(Command::StartOfLine)
                }
                (KeyCode::Char('a') | KeyCode::Char('A'), KeyModifiers::SHIFT) => {
                    self.mode = State::Insert;
                    Ok(Command::EndOfLine)
                }
                (KeyCode::Char('k'), KeyModifiers::NONE) => Ok(Command::Move(Direction::Up)),
                (KeyCode::Char('j'), KeyModifiers::NONE) => Ok(Command::Move(Direction::Down)),
                (KeyCode::Char('h'), KeyModifiers::NONE) => Ok(Command::Move(Direction::Left)),
                (KeyCode::Char('l'), KeyModifiers::NONE) => Ok(Command::Move(Direction::Right)),
                (KeyCode::Char('w'), KeyModifiers::NONE) => Ok(Command::NextWord),
                _ => anyhow::bail!("not yet implement"),
            },
            _ => anyhow::bail!("not yet implement"),
        }
    }

    fn insert_command(&mut self, event: KeyEvent) -> anyhow::Result<Command> {
        match event {
            KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            } => match (code, modifiers) {
                (KeyCode::Esc, KeyModifiers::NONE) => {
                    self.mode = State::Normal;
                    Ok(Command::Switch(State::Normal))
                }
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Command::Quit),
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => Ok(Command::Save),
                (KeyCode::Char('f'), KeyModifiers::CONTROL) => Ok(Command::Search),
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(Command::Insert(c))
                }
                (KeyCode::Tab, _) => Ok(Command::Insert('\t')),
                (KeyCode::Enter, _) => Ok(Command::Enter),
                (KeyCode::Backspace, _) => Ok(Command::Backspace),
                (KeyCode::Delete, _) => Ok(Command::Delete),
                (KeyCode::Up, _) => Ok(Command::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Command::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Command::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Command::Move(Direction::Right)),
                (KeyCode::Home, _) => Ok(Command::StartOfLine),
                (KeyCode::End, _) => Ok(Command::EndOfLine),
                // (KeyCode::Esc, KeyModifiers::NONE) => Ok(Command::Dismiss),
                _ => anyhow::bail!("not yet implement"),
            },
            _ => anyhow::bail!("not yet implement"),
        }
    }
}
