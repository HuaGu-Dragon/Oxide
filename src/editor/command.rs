use crate::{
    editor::{event::Command, ui::UiComponent, view::line::Line},
    terminal,
};

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    value: Line,
    width: u16,
    render: bool,
}

impl CommandBar {
    pub fn handle_edit_command(&mut self, command: Command) {
        match command {
            Command::Insert(c) => self.value.append_char(c),
            Command::Backspace | Command::Delete => self.value.delete_last(),
            _ => {}
        }
        self.set_render(true);
    }

    pub fn caret_pos_col(&self) -> usize {
        let max_width = self
            .prompt
            .len()
            .saturating_add(self.value.grapheme_count());
        std::cmp::min(self.width as usize, max_width)
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
        self.set_render(true);
    }

    pub fn get_value(&self) -> String {
        self.value.to_string()
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.set_render(true);
    }
}

impl UiComponent for CommandBar {
    fn set_render(&mut self, render: bool) {
        self.render = render
    }

    fn needs_render(&self) -> bool {
        self.render
    }

    fn set_size(&mut self, width: u16, _height: u16) {
        self.width = width
    }

    fn draw(&mut self, y: u16) -> anyhow::Result<()> {
        // TODO: use `width` instanceof `len`?
        let area = (self.width as usize).saturating_sub(self.prompt.len());

        let value_end = self.value.width();

        let value_start = value_end.saturating_sub(area);
        let message = format!(
            "{}{}",
            self.prompt,
            self.value.get_visable_graphemes(value_start..value_end)
        );

        terminal::print_at(0, y, true, message)
    }
}
