use crate::editor::Size;

pub trait UiComponent {
    fn set_render(&mut self, render: bool);

    fn needs_render(&self) -> bool;

    fn set_size(&mut self, width: u16, height: u16);

    fn resize(&mut self, size: Size) {
        self.set_size(size.width, size.height);
        self.set_render(true);
    }

    fn draw(&mut self) -> anyhow::Result<()>;

    fn render(&mut self) {
        if !self.needs_render() {
            return;
        }

        match self.draw() {
            Ok(()) => self.set_render(false),
            Err(_err) => {
                #[cfg(debug_assertions)]
                {
                    panic!("Could not render component: {_err:?}")
                }
            }
        }
    }
}
