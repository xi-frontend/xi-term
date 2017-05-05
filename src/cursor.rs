#[derive(Clone, Debug)]
pub struct Cursor {
    pub line: u64,
    pub column: u64,
}

impl From<(u64, u64)> for Cursor {
    fn from(pair: (u64, u64)) -> Self {
        Cursor {
            line: pair.0,
            column: pair.1,
        }
    }
}

impl Cursor {
    pub fn new() -> Self {
        Cursor::from((0, 0))
    }

    pub fn update(&mut self, cursor_pos: (u64, u64)) {
        let new_cursor = Cursor::from(cursor_pos);
        if self != &new_cursor {
            *self = new_cursor;
        }
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        self.line == other.line && self.column == other.column
    }
}

impl Eq for Cursor {}
