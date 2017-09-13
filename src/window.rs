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
            self.start = 1 + cursor.line - u64::from(self.size);
            self.dirty = true;
        }
    }

    pub fn resize(&mut self, height: u16, cursor: u64, last_line: u64) {
        if self.size == height {
            return;
        }

        // We have no way to know if the screen was resized from the top or the bottom, so we
        // balance the change between both end. Basically we want:
        //  1) new_height = new_end - new_start
        //  2) new_end - old_end = new_start - old_start
        // which leads to:
        //  1) new_end = (new_height + old_start + old_end) / 2
        //  2) new_start = (old_start + old_end - new_height) / 2
        let mut new_start = if u64::from(height) > self.start() + self.end() {
            //  Of course, new_start must be >=0, so we have this special case:
            0
        } else {
            (self.start() + self.end() - u64::from(height)) / 2
        };

        // Handle a first corner case where the previous operation gave us a window that end after
        // the last line. We don't want to waste this space, so we translate the window so that the
        // last line correspond to the end of the window.
        if new_start + u64::from(height) > last_line {
            if last_line < u64::from(height) {
                new_start = 0;
            } else {
                new_start = last_line - u64::from(height);
            }
        }

        // Handle a second corner case where the previous operations left the cursor out of scope.
        // We want to keep the cursor in the window.
        if cursor < new_start {
            new_start = cursor;
        } else if cursor > new_start + u64::from(height) {
            new_start = cursor - u64::from(height);
        }

        self.start = new_start;
        self.size = height;
        self.dirty = true;
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        u64::from(self.size) + self.start
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
        Some(((index - self.start) & u64::from(u16::max_value())) as u16)
    }
}
