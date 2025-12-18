#[derive(Clone, Copy, Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

#[derive(Default)]
pub struct Cursor {
    at: Location,
}

impl Cursor {
    // pub fn new(col: usize, row: usize) -> Self {
    //     Self {
    //         at: Location::new(col, row),
    //     }
    // }

    pub fn location(&self) -> Location {
        self.at
    }

    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.at
    }
}
