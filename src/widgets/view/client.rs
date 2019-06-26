use futures::Future;
use tokio::spawn;
use xrl;

use crate::core::{Command, RelativeMoveDistance, AbsoluteMovePoint};

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

    pub fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::Cancel => {/* Handled by TUI */}
            Command::OpenPrompt => {/* Handled by TUI */}
            Command::Quit => {/* Handled by TUI */}
            Command::SetTheme(_theme) => { /* Handled by Editor */ },
            Command::NextBuffer => { /* Handled by Editor */ },
            Command::PrevBuffer => { /* Handled by Editor */ },
            Command::Save(_view_id) => { /* Handled by Editor */ },
            Command::Open(_file) => { /* Handled by Editor */ },
            Command::ToggleLineNumbers => { /* Handled by View */ },
            Command::Back => self.back(),
            Command::Delete => self.delete(),    
            Command::Insert('\n') => self.insert_newline(),
            Command::Insert('\t') => self.insert_tab(),
            Command::Insert(c)    => self.insert(c),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::RelativeMove(x) => {
                match x.by {
                    RelativeMoveDistance::characters => {
                        if x.forward {
                            self.right()
                        } else {
                            self.left()
                        }
                    },
                    RelativeMoveDistance::pages => {
                        if x.forward {
                            self.page_down()
                        } else {
                            self.page_up()
                        }
                    },
                    RelativeMoveDistance::lines => {
                        if x.forward {
                            self.down()
                        } else {
                            self.up()
                        }
                    },
                    _ => unimplemented!()
                }
            }
            Command::AbsoluteMove(x) => {
                match x.to {
                    AbsoluteMovePoint::bol => self.line_start(),
                    AbsoluteMovePoint::eol => self.line_end(),
                    _ => unimplemented!()
                }
            }
        }
    }

    pub fn undo(&mut self) {
        let f = self.inner.undo(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn redo(&mut self) {
        let f = self.inner.redo(self.view_id).map_err(|_| ());
        spawn(f);
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

    pub fn line_start(&mut self) {
        let f = self.inner.line_start(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn line_end(&mut self) {
        let f = self.inner.line_end(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn delete(&mut self) {
        let f = self.inner.delete(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn back(&mut self) {
        let f = self.inner.backspace(self.view_id).map_err(|_| ());
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
