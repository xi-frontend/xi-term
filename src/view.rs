use std::io::Write;

use termion::{clear, cursor};

use cursor::Cursor;
use errors::*;
use line::Line;
use update::Update;

const TAB_LENGTH: u16 = 4;

#[derive(Clone)]
pub struct View {
    last_rev: u64,
    state: State,
    pub filepath: String,
    pub lines: Vec<Line>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone)]
enum State {
    /// The view did not change since last time it was rendered
    Clean,
    /// The lines changes since last time the view was rendered
    Lines,
    /// The cursor changed since last time the view was rendered
    Cursor,
    /// Both the lines and the cursor changed since last time the view was rendered
    All,
}

impl View {
    pub fn new(filepath: &str) -> View {
        View {
            last_rev: 0,
            filepath: filepath.to_owned(),
            lines: vec![],
            cursor: None,
            state: State::Clean,
        }
    }

    pub fn update_lines(&mut self, update: &Update) -> Result<()> {
        let mut lines = vec![];
        let mut index = 0;

        for operation in &update.operations {
            index = operation.apply(&self.lines, index, &mut lines)?;
        }

        self.lines = lines;
        self.set_dirty_lines();
        Ok(())
    }

    pub fn update_cursor(&mut self, cursor: &Cursor) {
        self.cursor = Some(cursor.clone());
        self.set_dirty_cursor();
    }

    /// Return true if the lines were updated since last time the view was rendered.
    fn dirty_lines(&self) -> bool {
        match self.state {
            State::All | State::Lines => true,
            _ => false,
        }
    }

    fn dirty_cursor(&self) -> bool {
        match self.state {
            State::All | State::Cursor => true,
            _ => false,
        }
    }

    fn set_dirty_lines(&mut self) {
        match self.state {
            State::Clean => self.state = State::Lines,
            State::Cursor => self.state = State::All,
            _ => {}
        }
    }

    fn set_dirty_cursor(&mut self) {
        match self.state {
            State::Clean => self.state = State::Cursor,
            State::Lines => self.state = State::All,
            _ => {}
        }
    }

    fn set_clean_lines(&mut self) {
        match self.state {
            State::All => self.state = State::Cursor,
            State::Lines => self.state = State::Clean,
            _ => {}
        }
    }

    fn set_clean_cursor(&mut self) {
        match self.state {
            State::All => self.state = State::Lines,
            State::Cursor => self.state = State::Clean,
            _ => {}
        }
    }

    pub fn render<W: Write>(&mut self, w: &mut W, height: u16) -> Result<()> {
        if self.dirty_lines() {
            write!(w, "{}{}", cursor::Goto(1, 1), clear::All)
                .chain_err(|| ErrorKind::DisplayError)?;

            self.render_lines(w, height)?;
            self.set_clean_lines();
        }

        if self.dirty_cursor() {
            self.render_cursor(w)?;
            self.set_clean_cursor();
        }
        Ok(())
    }

    fn render_lines<W: Write>(&self, w: &mut W, height: u16) -> Result<()> {
        let mut invalid_lines: usize = 0;
        for (lineno, line) in self.lines.iter().enumerate() {
            if line.is_valid {
                // Lines are drawn with fake cursors.
                // We set the actual cursor later, redrawing the line in the process.
                line.render(w, (lineno - invalid_lines) as u16 + 1, None)?;
            } else {
                invalid_lines += 1;
            }
            if lineno > invalid_lines && (lineno - invalid_lines) == height as usize {
                break;
            }
        }
        Ok(())
    }

    pub fn render_cursor<W: Write>(&self, w: &mut W) -> Result<()> {
        if let Some(cursor) = self.cursor.as_ref() {
            if cursor.line as usize <= self.lines.len() {
                // Redraw the line without the fake cursor
                let line = self.lines
                    .get(cursor.line as usize)
                    .and_then(|line| if line.is_valid { Some(line) } else { None })
                    .ok_or_else(|| {
                        error!("No valid line at cursor index {}", cursor.line);
                        ErrorKind::DisplayError
                    })?;
                line.render(w, cursor.line + 1, Some(cursor))?;

                // Draw the cursor. The trick is that some characters are larger than others.
                //
                // For the moment, we only handle tabs, and we assume the terminal has tabstops of
                // TAB_LENGTH.
                let column = line.text
                    .as_ref()
                    .map(|s| &**s)
                    .unwrap_or("")
                    .chars()
                    .take(cursor.column as usize)
                    .fold(0, |acc, c| {
                        if c == '\t' {
                            acc + TAB_LENGTH - (acc % TAB_LENGTH)
                        } else {
                            acc + 1
                        }
                    });
                let cursor_pos = cursor::Goto(column as u16 + 1, cursor.line + 1);
                write!(w, "{}", cursor_pos)
                    .chain_err(|| ErrorKind::DisplayError)?;
                w.flush().chain_err(|| ErrorKind::DisplayError)?;
            } else {
                error!("Cursor is on line {} but we have only {} lines",
                       cursor.line, self.lines.len());
                bail!(ErrorKind::DisplayError)
            }
        } else {
            warn!("No cursor to render");
        }
        Ok(())
    }
}
