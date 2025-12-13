use anyhow::Context;

use crate::editor::Editor;

mod editor;

fn main() -> anyhow::Result<()> {
    let editor = Editor::new();
    editor.run().context("running exide")?;

    Ok(())
}
