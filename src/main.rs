use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use crate::editor::Editor;

mod editor;
mod terminal;
mod view;

#[derive(Parser)]
pub struct Cli {
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    Editor::new().run().context("run the editor")?;

    Ok(())
}
