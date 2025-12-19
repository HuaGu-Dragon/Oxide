use anyhow::{Ok, anyhow};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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
    Resize(u16, u16),
    Quit,
    Insert(char),
    Delete,
    Backspace,
    Enter,
    Save,
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
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(c)),
                (KeyCode::Tab, _) => Ok(Self::Insert('\t')),
                (KeyCode::Enter, _) => Ok(Self::Enter),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Delete, _) => Ok(Self::Delete),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                _ => Err(anyhow!("Not yet implement")),
            },
            Event::Mouse(_mouse_event) => Err(anyhow!("Not yet implement")),
            Event::Paste(_) => Err(anyhow!("Not yet implement")),
            Event::Resize(width, height) => Ok(Self::Resize(width, height)),
            _ => Err(anyhow!("Not yet implement")),
        }
    }
}
