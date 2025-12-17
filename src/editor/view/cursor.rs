use crate::editor::view::location::Location;

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

    pub fn at(&self) -> &Location {
        &self.at
    }

    pub fn set_pos(&mut self, x: usize, y: usize) {
        self.at.set_pos(x, y);
    }
}
