use futures::Future;
use tokio::spawn;
use xrl;
use serde_json::Value;

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
            Command::FindUnderExpand => { /* Handled by View */ },
            Command::CopySelection => { /* Handled by View */ },
            Command::CutSelection => { /* Handled by View */ },
            Command::Paste => { /* Handled by View */ },
            Command::Back => self.back(),
            Command::Delete => self.delete(),    
            Command::Insert('\n') => self.insert_newline(),
            Command::Insert('\t') => self.insert_tab(),
            Command::Insert(c)    => self.insert(c),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::CursorExpandLines(dir) => self.cursor_expand_line(dir.forward),
            Command::CloseCurrentView => self.close(),
            Command::RelativeMove(x) => {
                match x.by {
                    RelativeMoveDistance::characters => {
                        if x.forward {
                            self.right(x.extend)
                        } else {
                            self.left(x.extend)
                        }
                    },
                    RelativeMoveDistance::words | RelativeMoveDistance::word_ends => {
                        if x.forward {
                            self.word_right(x.extend)
                        } else {
                            self.word_left(x.extend)
                        }
                    },
                    RelativeMoveDistance::pages => {
                        if x.forward {
                            self.page_down(x.extend)
                        } else {
                            self.page_up(x.extend)
                        }
                    },
                    RelativeMoveDistance::lines => {
                        if x.forward {
                            self.down(x.extend)
                        } else {
                            self.up(x.extend)
                        }
                    },
                    _ => unimplemented!()
                }
            }
            Command::AbsoluteMove(x) => {
                match x.to {
                    AbsoluteMovePoint::bol => self.line_start(x.extend),
                    AbsoluteMovePoint::eol => self.line_end(x.extend),
                    AbsoluteMovePoint::bof => self.document_begin(x.extend),
                    AbsoluteMovePoint::eof => self.document_end(x.extend),
                    AbsoluteMovePoint::line(line) => self.goto_line(line),
                    _ => unimplemented!()
                }
            }
        }
    }


    pub fn close(&mut self) {
        let f = self.inner.close_view(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn find_under_expand_next(&mut self) {
        let f = self.inner
                    .find_next(self.view_id, true, false, xrl::ModifySelection::Add)
                    .map_err(|_| ());
        spawn(f);        
    }

    pub fn find_under_expand(&mut self) {
        let f = self.inner.edit_notify(self.view_id, "selection_for_find", Some(json!({"case_sensitive": true})))
                    .map_err(|_| ());
        spawn(f);
    }

    pub fn copy(&mut self) -> impl Future<Item = Value, Error = xrl::ClientError> {
        self.inner.copy(self.view_id)
    }

    pub fn cut(&mut self) -> impl Future<Item = Value, Error = xrl::ClientError> {
        self.inner.cut(self.view_id)
    }

    pub fn paste(&mut self, content: &str) {
        let f = self.inner.paste(self.view_id, content).map_err(|_| ());
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

    pub fn down(&mut self, extend: bool) {
        if extend {
            let f = self.inner.down_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.down(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn up(&mut self, extend: bool) {
        if extend {
            let f = self.inner.up_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.up(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn right(&mut self, extend: bool) {
        if extend {
            let f = self.inner.right_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {
            let f = self.inner.right(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn left(&mut self, extend: bool) {
        if extend {
            let f = self.inner.left_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {
            let f = self.inner.left(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn word_right(&mut self, extend: bool) {
        if extend {
            let f = self.inner.move_word_right_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {
            let f = self.inner.move_word_right(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn word_left(&mut self, extend: bool) {
        if extend {
            let f = self.inner.move_word_left_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {
            let f = self.inner.move_word_left(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn page_down(&mut self, extend: bool) {
        if extend {
            let f = self.inner.page_down_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.page_down(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn page_up(&mut self, extend: bool) {
        if extend {
            let f = self.inner.page_up_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.page_up(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn line_start(&mut self, extend: bool) {
        if extend {
            let f = self.inner.line_start_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.line_start(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn line_end(&mut self, extend: bool) {
        if extend {
            let f = self.inner.line_end_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.line_end(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn document_begin(&mut self, extend: bool) {
        if extend {
            let f = self.inner.document_begin_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.document_begin(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn document_end(&mut self, extend: bool) {
        if extend {
            let f = self.inner.document_end_sel(self.view_id).map_err(|_| ());
            spawn(f);
        } else {        
            let f = self.inner.document_end(self.view_id).map_err(|_| ());
            spawn(f);
        }
    }

    pub fn goto_line(&mut self, line: u64) {
        let f = self.inner.goto_line(self.view_id, line).map_err(|_| ());
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

    pub fn collapse_selections(&mut self) {
        let f = self.inner.collapse_selections(self.view_id).map_err(|_| ());
        spawn(f);
    }

    pub fn cursor_expand_line(&mut self, forward: bool) {
        let command = if forward { "add_selection_below" } else { "add_selection_above" };
        let f = self.inner.edit_notify(self.view_id, command, None as Option<Value>)
                    .map_err(|_| ());
        spawn(f);
    }
}
