use cursor::Cursor;

#[derive(Clone, Debug)]
pub struct Window {
    start: u64,
    size: u16,
    dirty: bool,
}

impl Window {
    pub fn new() -> Self {
        Window {
            start: 0,
            size: 0,
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn update(&mut self, cursor: &Cursor) {
        if cursor.line < self.start() {
            self.start = cursor.line;
            self.dirty = true;
        } else if cursor.line >= self.end() {
            self.start = 1 + cursor.line - self.size as u64;
            self.dirty = true;
        }
    }

    pub fn resize(&mut self, new_size: u16) {
        if self.size != new_size {
            self.size = new_size;
            self.dirty = true;
        }
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.size as u64 + self.start
    }

    pub fn is_within_window(&self, index: u64) -> bool {
        if self.start <= index && index < self.end() {
            return true;
        }
        false
    }

    pub fn offset(&self, index: u64) -> Option<u16> {
        if !self.is_within_window(index) {
            return None;
        }
        Some(((index - self.start) & u16::max_value() as u64) as u16)
    }
}
