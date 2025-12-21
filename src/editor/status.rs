use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{editor::DocumentStatus, terminal};

pub struct StatusBar {
    status: DocumentStatus,
    render: bool,
    margin_bottom: u16,
    width: u16,
    position_y: u16,
}

impl StatusBar {
    pub fn new(margin_bottom: u16) -> Self {
        let (width, height) = terminal::size().unwrap_or_default();
        Self {
            status: DocumentStatus::default(),
            render: true,
            margin_bottom,
            width,
            position_y: height.saturating_sub(margin_bottom).saturating_sub(1),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.position_y = height.saturating_sub(self.margin_bottom).saturating_sub(1);
        self.render = true;
    }

    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.status != status {
            self.status = status;
            self.render = true;
        }
    }

    pub fn render(&mut self) {
        if !self.render {
            return;
        }

        self.render = false;

        let modified_indicator = self.status.modified_indicator();
        let line_count = self.status.line_count();

        let beginning = format!("{} - {line_count} {modified_indicator}", self.status.file);

        let position_indicator = self.status.position_indicator();
        let reminder_len = (self.width as usize).saturating_sub(
            beginning
                .graphemes(true)
                .map(|graphme| graphme.width())
                .sum(),
        );

        let status = format!("{beginning}{position_indicator:>reminder_len$}");

        // status.truncate(self.width as usize);
        let res = terminal::print_inverted_at(0, self.position_y, true, status);
        debug_assert!(res.is_ok());
    }
}
