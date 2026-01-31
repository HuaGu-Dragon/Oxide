use anyhow::{Ok, anyhow};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::editor::{Size, control::State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Move(Direction),
    Resize(Size),
    Quit,
    Insert(char),
    Delete,
    Backspace,
    StartOfLine,
    EndOfLine,
    Enter,
    Save,
    Search,
    Dismiss,
    Switch(State),
}

impl TryFrom<Event> for Command {
    type Error = anyhow::Error;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::FocusGained => Err(anyhow!("Not yet implement")),
            Event::FocusLost => Err(anyhow!("Not yet implement")),
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => Ok(Self::Save),
                (KeyCode::Char('f'), KeyModifiers::CONTROL) => Ok(Self::Search),
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(c)),
                (KeyCode::Tab, _) => Ok(Self::Insert('\t')),
                (KeyCode::Enter, _) => Ok(Self::Enter),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Delete, _) => Ok(Self::Delete),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::Home, _) => Ok(Self::StartOfLine),
                (KeyCode::End, _) => Ok(Self::EndOfLine),
                (KeyCode::Esc, KeyModifiers::NONE) => Ok(Self::Dismiss),
                _ => Err(anyhow!("Not yet implement")),
            },
            Event::Mouse(_mouse_event) => Err(anyhow!("Not yet implement")),
            Event::Paste(_) => Err(anyhow!("Not yet implement")),
            Event::Resize(width, height) => Ok(Self::Resize(Size { width, height })),
            _ => Err(anyhow!("Not yet implement")),
        }
    }
}
