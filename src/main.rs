use anyhow::Context;

use crate::editor::Editor;

mod editor;
mod terminal;

fn main() -> anyhow::Result<()> {
    Editor::new().run().context("run the editor")?;

    Ok(())
}
