use super::view::Cursor;

#[derive(Clone, Debug)]
pub struct Window {
    start: u64,
    size: u16,
}

impl Window {
    pub fn new() -> Self {
        Window { start: 0, size: 0 }
    }

    pub fn set_cursor(&mut self, cursor: &Cursor) {
        info!("Setting cursor to {:?}", cursor);
        if cursor.line < self.start() {
            self.start = cursor.line;
        } else if cursor.line >= self.end() {
            self.start = 1 + cursor.line - u64::from(self.size);
        }
        info!("new window: {:?}", self);
    }

    pub fn resize(&mut self, height: u16) {
        self.size = height;
    }

    pub fn update(&mut self, cursor: u64, nb_line: u64) {
        info!(
            "resizing window: height={}, cursor={}, nb_line={}",
            self.size, cursor, nb_line
        );

        // We have no way to know if the screen was resized from the top or the bottom, so we
        // balance the change between both end. Basically we want:
        //  1) new_height = new_end - new_start
        //  2) new_end - old_end = new_start - old_start
        // which leads to:
        //  1) new_end = (new_height + old_start + old_end) / 2
        //  2) new_start = (old_start + old_end - new_height) / 2
        let mut new_start = if u64::from(self.size) > self.start() + self.end() {
            //  Of course, new_start must be >=0, so we have this special case:
            0
        } else {
            (self.start() + self.end() - u64::from(self.size)) / 2
        };

        // Handle a first corner case where the previous operation gave us a window that end after
        // the last line. We don't want to waste this space, so we translate the window so that the
        // last line correspond to the end of the window.
        if new_start + u64::from(self.size) > nb_line {
            if nb_line < u64::from(self.size) {
                new_start = 0;
            } else {
                new_start = nb_line - u64::from(self.size);
            }
        }

        // Handle a second corner case where the previous operations left the cursor out of scope.
        // We want to keep the cursor in the window.
        if cursor < new_start {
            new_start = cursor;
        } else if cursor > new_start + u64::from(self.size) {
            new_start = cursor - u64::from(self.size);
        }

        self.start = new_start;
        info!("done resizing the window: {:?}", self);
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
}
