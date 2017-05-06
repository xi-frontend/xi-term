use std::io::Write;
use std::collections::HashMap;

use termion::clear;
use termion::cursor;

use cache::LineCache;
use cursor::Cursor;
use errors::*;
use style::Style;
use update::Update;
use window::Window;

const TAB_LENGTH: u16 = 4;

#[derive(Clone, Debug)]
pub struct View {
    last_rev: u64,
    pub filepath: String,
    cache: LineCache,
    cursor: Cursor,
    window: Window,
    styles: HashMap<u16, Style>,
}

impl View {
    pub fn new(filepath: &str) -> View {
        View {
            last_rev: 0,
            filepath: filepath.to_owned(),
            cache: LineCache::new(),
            cursor: Cursor::new(),
            window: Window::new(),
            styles: HashMap::new(),
        }
    }

    pub fn set_style(&mut self, style: Style) {
        self.styles.insert(style.id, style);
    }

    pub fn update_lines(&mut self, update: &Update) -> Result<()> {
        self.cache.update(update)
    }

    pub fn update_cursor(&mut self, cursor_pos: (u64, u64)) {
        self.cursor.update(cursor_pos);
        self.window.update(&self.cursor.clone());
    }

    pub fn get_window(&self) -> (u64, u64) {
        (self.window.start(), self.window.end())
    }

    pub fn render<W: Write>(&mut self, w: &mut W, height: u16) -> Result<()> {
        self.window.resize(height);

        if self.cache.is_dirty() || self.window.is_dirty() {
            write!(w, "{}{}", cursor::Goto(1, 1), clear::All)
                .chain_err(|| ErrorKind::DisplayError)?;

            self.render_lines(w)?;
            self.cache.mark_clean();
            self.window.mark_clean();
        }

        self.render_cursor(w)?;

        Ok(())
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn render_lines<W: Write>(&self, w: &mut W) -> Result<()> {
        debug!("Rendering lines");

        // Get the lines that are within the displayed window
        let lines = self.cache
            .lines()
            .iter()
            .skip(self.window.start() as usize)
            .take(self.window.size() as usize);

        // Draw the valid lines within this range
        for (lineno, line) in lines.enumerate() {
            if !line.is_valid {
                continue;
            }

            // Get the line vertical offset so that we know where to draw it.
            let line_pos = self.window
                .offset(self.window.start() + lineno as u64)
                .ok_or_else(|| {
                    error!("Could not find line position within the window");
                    ErrorKind::DisplayError
                })?;

            line.render(w, line_pos + 1)?;
        }
        Ok(())
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn render_cursor<W: Write>(&self, w: &mut W) -> Result<()> {
        debug!("Rendering cursor");
        if !self.window.is_within_window(self.cursor.line) {
            error!("Cursor is on line {} which is not within the displayed window", self.cursor.line);
            bail!(ErrorKind::DisplayError)
        }

        // Get the line that has the cursor
        let line = self.cache
            .lines()
            .get(self.cursor.line as usize)
            .and_then(|line| if line.is_valid { Some(line) } else { None })
            .ok_or_else(|| {
                error!("No valid line at cursor index {}", self.cursor.line);
                ErrorKind::DisplayError
            })?;

        // Get the line vertical offset so that we know where to draw it.
        let line_pos = self.window
            .offset(self.cursor.line)
            .ok_or_else(|| {
                error!("Could not find line position within the window: {:?}", line);
                ErrorKind::DisplayError
            })?;

        // Calculate the cursor position on the line. The trick is that we know the position within
        // the string, but characters may have various lengths. For the moment, we only handle
        // tabs, and we assume the terminal has tabstops of TAB_LENGTH. We consider that all the
        // other characters have a width of 1.
        let column = line.text
            .chars()
            .take(self.cursor.column as usize)
            .fold(0, add_char_width);

        // Draw the cursor
        let cursor_pos = cursor::Goto(column as u16 + 1, line_pos + 1);
        write!(w, "{}", cursor_pos)
            .chain_err(|| ErrorKind::DisplayError)?;
        debug!("Cursor set at line {} column {}", line_pos, column);
        w.flush().chain_err(|| ErrorKind::DisplayError)?;

        Ok(())
    }
}

fn add_char_width(acc: u16, c: char) -> u16 {
    if c == '\t' {
        acc + TAB_LENGTH - (acc % TAB_LENGTH)
    } else {
        acc + 1
    }
}
