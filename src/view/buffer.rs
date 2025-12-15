use std::{ops::Deref, path::PathBuf};

pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn load(&mut self, path: PathBuf) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            self.lines = contents.lines().map(String::from).collect();
        }
    }
}

impl Deref for Buffer {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        self.lines.as_ref()
    }
}
