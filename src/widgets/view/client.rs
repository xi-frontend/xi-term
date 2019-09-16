use futures::Future;
use tokio::spawn;
use xrl;

pub struct Client {
    inner: xrl::Client,
    view_id: xrl::ViewId,
}

impl Client {
    pub fn new(client: xrl::Client, view_id: xrl::ViewId) -> Self {
        Client {
            inner: client,
            view_id,
        }
    }

    pub fn insert(&mut self, character: char) {
        let f = self.inner.char(self.view_id, character).map_err(|_| ());
        spawn(f);
    }

    pub fn insert_newline(&mut self) {
        let f = self.inner.insert_newline(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn insert_tab(&mut self) {
        let f = self.inner.insert_tab(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn scroll(&mut self, start: u64, end: u64) {
        let f = self.inner.scroll(self.view_id, start, end).map_err(|_| ());
        spawn(f);
    }

    pub fn down(&mut self) {
        let f = self.inner.down(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn up(&mut self) {
        let f = self.inner.up(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn right(&mut self) {
        let f = self.inner.right(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn left(&mut self) {
        let f = self.inner.left(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn page_down(&mut self) {
        let f = self.inner.page_down(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn page_up(&mut self) {
        let f = self.inner.page_up(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn home(&mut self) {
        let f = self.inner.line_start(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn end(&mut self) {
        let f = self.inner.line_end(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn delete(&mut self) {
        let f = self.inner.delete(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn backspace(&mut self) {
        let f = self.inner.backspace(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn undo(&mut self) {
        let f = self.inner.undo(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn redo(&mut self) {
        let f = self.inner.redo(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn select_all(&mut self) {
        let f = self.inner.select_all(self.view_id).map_err(|_| ());
        spawn(f);
    }
    pub fn save(&mut self, file: &str) {
        let f = self.inner.save(self.view_id, file).map_err(|_| ());
        spawn(f);
    }

    pub fn click(&mut self, line: u64, column: u64) {
        let f = self
            .inner
            .click_point_select(self.view_id, line, column)
            .map_err(|_| ());
        spawn(f);
    }

    pub fn drag(&mut self, line: u64, column: u64) {
        let f = self.inner.drag(self.view_id, line, column).map_err(|_| ());
        spawn(f);
    }
}
