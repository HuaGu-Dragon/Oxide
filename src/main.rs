use std::path::PathBuf;

use clap::Parser;

use crate::editor::Editor;

mod editor;
mod terminal;
mod view;

#[derive(Parser)]
pub struct Cli {
    path: Option<PathBuf>,
}

fn main() {
    Editor::new().unwrap().run()
}
