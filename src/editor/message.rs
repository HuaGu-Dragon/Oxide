use crate::{editor::ui::UiComponent, terminal};

#[derive(Default)]
pub struct MessageBar {
    message: String,
    render: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, msg: String) {
        if msg != self.message {
            self.message = msg;
            self.set_render(true);
        }
    }
}

impl UiComponent for MessageBar {
    fn set_render(&mut self, render: bool) {
        self.render = render;
    }

    fn needs_render(&self) -> bool {
        self.render
    }

    fn set_size(&mut self, _width: usize, _height: usize) {}

    fn draw(&mut self) -> anyhow::Result<()> {
        let (_, rows) = terminal::size()?;
        terminal::print_at(0, rows.saturating_sub(1), true, &self.message)
    }
}
