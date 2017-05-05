#[derive(Clone, Debug)]
pub struct Cursor {
    pub line: u16,
    pub column: u16,
}

impl From<(u64, u64)> for Cursor {
    fn from(pair: (u64, u64)) -> Self {
        Cursor {
            line: (pair.0 & u16::max_value() as u64) as u16,
            column: (pair.1 & u16::max_value() as u64) as u16,
        }
    }
}

