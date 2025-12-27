use unicode_width::UnicodeWidthStr;

use crate::{
    editor::{DocumentStatus, ui::UiComponent},
    terminal,
};

#[derive(Default)]
pub struct StatusBar {
    status: DocumentStatus,
    render: bool,
    width: u16,
}

impl StatusBar {
    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.status != status {
            self.status = status;
            self.render = true;
        }
    }
}

impl UiComponent for StatusBar {
    fn set_render(&mut self, render: bool) {
        self.render = render;
    }

    fn needs_render(&self) -> bool {
        self.render
    }

    fn set_size(&mut self, width: u16, _height: u16) {
        self.width = width;
    }

    fn draw(&mut self, y: u16) -> anyhow::Result<()> {
        let modified_indicator = self.status.modified_indicator();
        let line_count = self.status.line_count();

        let beginning = format!("{} - {line_count} {modified_indicator}", self.status.file);
        let position_indicator = self.status.position_indicator();
        let reminder_len = (self.width as usize).saturating_sub(beginning.width());

        let status = format!("{beginning}{position_indicator:>reminder_len$}");

        // status.truncate(self.width as usize);
        terminal::print_inverted_at(0, y, true, status)
    }
}
