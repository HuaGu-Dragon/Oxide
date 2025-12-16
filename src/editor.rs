use std::panic;

use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};

use crate::{Cli, terminal, view::View};

pub struct Editor {
    should_quit: bool,
    /// The current position of the caret in the editor.
    pos: Position,
    view: View,
}

// TODO: move it into the terminal, because it's not editor specific
// editor need the (col, row) of the current buffer not this.
struct Position {
    x: u16,
    y: u16,
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
            pos: Position { x: 0, y: 0 },
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
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}")
                    }
                }
            }
        }
    }

    fn evalute_event(&mut self, event: Event) {
        match event {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(key_event) => match key_event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    self.should_quit = true;
                }
                KeyEvent {
                    code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                    kind: KeyEventKind::Press,
                    ..
                } => self.move_point(key_event.code),
                _ => {}
            },
            Event::Mouse(_mouse_event) => {}
            Event::Paste(_) => {}
            Event::Resize(width, height) => self.view.resize(width, height),
        }
    }

    fn move_point(&mut self, code: KeyCode) {
        let pos = &mut self.pos;
        let (cols, rows) = self.view.size();

        // TODO: A Cursor struct to handle cursor position and size, like move saturating_left()
        match code {
            KeyCode::Up => {
                pos.y = pos.y.saturating_sub(1);
            }
            KeyCode::Down => {
                pos.y = pos.y.saturating_add(1).min(rows.saturating_sub(1));
            }
            KeyCode::Left => {
                pos.x = pos.x.saturating_sub(1);
            }
            KeyCode::Right => {
                pos.x = pos.x.saturating_add(1).min(cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn refresh_screen(&mut self) {
        let _ = terminal::hide_caret();

        self.view.render();

        let _ = terminal::move_caret(self.pos.x, self.pos.y);
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
