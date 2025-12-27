use std::time::{Duration, Instant};

use crate::{editor::ui::UiComponent, terminal};

const MESSAGE_DURATION: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct MessageBar {
    message: Message,
    render: bool,
    clear_after_expiry: bool,
}

struct Message {
    instant: Instant,
    text: String,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            instant: Instant::now(),
            text: Default::default(),
        }
    }
}

impl Message {
    fn is_expired(&self) -> bool {
        self.instant.elapsed() > MESSAGE_DURATION
    }
}

impl MessageBar {
    pub fn update_message(&mut self, msg: String) {
        self.message = Message {
            instant: Instant::now(),
            text: msg,
        };
        self.clear_after_expiry = false;
        self.set_render(true);
    }
}

impl UiComponent for MessageBar {
    fn set_render(&mut self, render: bool) {
        self.render = render;
    }

    fn needs_render(&self) -> bool {
        (!self.clear_after_expiry && self.message.is_expired()) || self.render
    }

    fn set_size(&mut self, _width: u16, _height: u16) {}

    fn draw(&mut self, y: u16) -> anyhow::Result<()> {
        if self.message.is_expired() {
            self.clear_after_expiry = true;
        }
        let message = if self.message.is_expired() {
            ""
        } else {
            &self.message.text
        };

        terminal::print_at(0, y, true, message)
    }
}
