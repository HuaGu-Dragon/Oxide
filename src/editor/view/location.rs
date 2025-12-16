#[derive(Default)]
pub struct Location {
    col: usize,
    row: usize,
}

impl Location {
    // pub fn new(col: usize, row: usize) -> Self {
    //     Self { col, row }
    // }

    pub fn pos(&self) -> (usize, usize) {
        (self.col, self.row)
    }

    pub fn pos_mut(&mut self) -> (&mut usize, &mut usize) {
        (&mut self.col, &mut self.row)
    }

    pub fn subtract(&self, other: &Location) -> (u16, u16) {
        let col = self.col.saturating_sub(other.col);
        let row = self.row.saturating_sub(other.row);
        (col as u16, row as u16)
    }
}
