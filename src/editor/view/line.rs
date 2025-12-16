use std::ops::{Deref, Range};

pub struct Line {
    text: String,
}

impl Line {
    pub fn get(&self, range: Range<usize>) -> &str {
        let start = range.start.min(self.text.len());
        let end = range.end.min(self.text.len());
        &self.text[start..end]
    }
}

impl From<String> for Line {
    fn from(value: String) -> Self {
        Line { text: value }
    }
}

impl From<&str> for Line {
    fn from(value: &str) -> Self {
        Line {
            text: String::from(value),
        }
    }
}

impl Deref for Line {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

#[test]
fn test_line_range() {
    let line = Line::from("Hello, world!");
    assert_eq!(line.len(), 13);
    assert_eq!(line.get(0..5), "Hello");
    assert_eq!(line.get(7..13), "world!");
    assert_eq!(line.get(13..18), "");
}
