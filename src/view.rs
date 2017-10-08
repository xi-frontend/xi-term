use std::io::Write;
use std::collections::HashMap;

use xrl::{Line, Update};
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

use cache::LineCache;
use window::Window;
use xrl::Style;
use style::{reset_style, set_style};

use errors::*;

const TAB_LENGTH: u16 = 4;

#[derive(Debug, Default)]
pub struct Cursor {
    pub line: u64,
    pub column: u64,
}


#[derive(Debug)]
pub struct View {
    cache: LineCache,
    cursor: Cursor,
    window: Window,
}

impl View {
    pub fn new() -> View {
        View {
            cache: LineCache::new(),
            cursor: Default::default(),
            window: Window::new(),
        }
    }

    pub fn update_cache(&mut self, update: Update) {
        info!("updating cache");
        self.cache.update(update)
    }

    pub fn set_cursor(&mut self, line: u64, column: u64) {
        self.cursor = Cursor {
            line: line,
            column: column,
        };
        self.window.update(&self.cursor);
    }

    pub fn render<W: Write>(&mut self, w: &mut W, styles: &HashMap<u64, Style>) -> Result<()> {
        self.render_lines(w, styles)?;
        self.render_cursor(w);
        Ok(())
    }

    pub fn resize(&mut self, height: u16) {
        if self.cursor.line < self.cache.invalid_before {
            error!(
                "cursor is on line {} but there are {} invalid lines in cache.",
                self.cursor.line,
                self.cache.invalid_before
            );
            return;
        }
        let cursor_line = self.cursor.line - self.cache.invalid_before;
        let nb_lines = self.cache.lines.len() as u64;
        self.window.resize(height, cursor_line, nb_lines);
    }

    pub fn click(&self, x: u64, y: u64) -> (u64, u64) {
        let lineno = x + self.cache.invalid_before + self.window.start();
        if let Some(line) = self.cache.lines.get(x as usize) {
            if y == 0 {
                return (lineno, 0);
            }
            let mut text_len: u16 = 0;
            for (idx, c) in line.text.chars().enumerate() {
                text_len = add_char_width(text_len, c);
                if u64::from(text_len) >= y {
                    return (lineno as u64, idx as u64 + 1);
                }
            }
            return (lineno, line.text.len() as u64 + 1);
        } else {
            warn!("no line at index {} found in cache", x);
            return (x, y);
        }
    }

    fn render_lines<W: Write>(&self, w: &mut W, styles: &HashMap<u64, Style>) -> Result<()> {
        debug!("rendering lines");
        trace!("current cache\n{:?}", self.cache);

        // Get the lines that are within the displayed window
        let lines = self.cache
            .lines
            .iter()
            .skip(self.window.start() as usize)
            .take(self.window.size() as usize);

        // Draw the valid lines within this range
        for (lineno, line) in lines.enumerate() {
            self.render_line(w, line, lineno, styles);
        }
        Ok(())
    }

    fn render_line<W: Write>(
        &self,
        w: &mut W,
        line: &Line,
        lineno: usize,
        styles: &HashMap<u64, Style>,
    ) {
        let text = self.add_styles(styles, line);
        if let Err(e) = write!(w, "{}{}{}", Goto(1, lineno as u16 + 1), ClearLine, &text) {
            error!("failed to render line: {}", e);
        }
    }

    fn add_styles(&self, styles: &HashMap<u64, Style>, line: &Line) -> String {
        let mut text = line.text.clone();
        if line.styles.is_empty() {
            return text;
        }
        let mut style_sequences = self.get_style_sequences(styles, line);
        for style in style_sequences.drain(..) {
            debug!("inserting style: {:?}", style);
            if style.0 >= text.len() {
                text.push_str(&style.1);
            } else {
                text.insert_str(style.0, &style.1);
            }
        }
        debug!("styled line: {:?}", text);
        text
    }

    fn get_style_sequences(
        &self,
        styles: &HashMap<u64, Style>,
        line: &Line,
    ) -> Vec<(usize, String)> {
        let mut style_sequences: Vec<(usize, String)> = Vec::new();
        let mut prev_style_end: usize = 0;
        for style_def in &line.styles {
            let start_idx = if style_def.offset >= 0 {
                (prev_style_end + style_def.offset as usize)
            } else {
                // FIXME: does that actually work?
                (prev_style_end - ((-style_def.offset) as usize))
            };
            let end_idx = start_idx + style_def.length as usize;
            prev_style_end = end_idx;

            if let Some(style) = styles.get(&style_def.style_id) {
                let start_sequence = match set_style(style) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("could not get CSI sequence to set style {:?}: {}", style, e);
                        continue;
                    }
                };
                let end_sequence = match reset_style(style) {
                    Ok(s) => s,
                    Err(e) => {
                        error!(
                            "could not get CSI sequence to reset style {:?}: {}",
                            style,
                            e
                        );
                        continue;
                    }
                };
                style_sequences.push((start_idx, start_sequence));
                style_sequences.push((end_idx, end_sequence));
            } else {
                error!(
                    "no style ID {} found. Not applying style.",
                    style_def.style_id
                );
            };
        }
        // Note that we sort the vector in *reverse* order, so that we apply style starting from
        // the end of the line, and we don't have to worry about the indices changing.
        style_sequences.sort_by(|a, b| a.0.cmp(&b.0));
        style_sequences.reverse();
        trace!("{:?}", style_sequences);
        style_sequences
    }

    pub fn render_cursor<W: Write>(&self, w: &mut W) {
        info!("rendering cursor");
        if self.cache.is_empty() {
            info!("cache is empty, rendering cursor at the top left corner");
            if let Err(e) = write!(w, "{}", Goto(1, 1)) {
                error!("failed to render cursor: {}", e);
            }
            return;
        }

        if self.cursor.line < self.cache.invalid_before {
            error!(
                "the cursor is on line {} which is marked invalid in the cache",
                self.cursor.line
            );
            return;
        }
        // Get the line that has the cursor
        let line_idx = self.cursor.line - self.cache.invalid_before;
        let line = match self.cache.lines.get(line_idx as usize) {
            Some(line) => line,
            None => {
                error!("no valid line at cursor index {}", self.cursor.line);
                return;
            }
        };

        if line_idx < self.window.start() {
            error!(
                "the line that has the cursor (nb={}, cache_idx={}) not within the displayed window ({:?})",
                self.cursor.line,
                line_idx,
                self.window
            );
            return;
        }
        // Get the line vertical offset so that we know where to draw it.
        let line_pos = line_idx - self.window.start();

        // Calculate the cursor position on the line. The trick is that we know the position within
        // the string, but characters may have various lengths. For the moment, we only handle
        // tabs, and we assume the terminal has tabstops of TAB_LENGTH. We consider that all the
        // other characters have a width of 1.
        let column = line.text
            .chars()
            .take(self.cursor.column as usize)
            .fold(0, add_char_width);

        // Draw the cursor
        let cursor_pos = Goto(column as u16 + 1, line_pos as u16 + 1);
        if let Err(e) = write!(w, "{}", cursor_pos) {
            error!("failed to render cursor: {}", e);
        }
        info!("Cursor rendered at ({}, {})", line_pos, column);
    }
}

fn add_char_width(acc: u16, c: char) -> u16 {
    if c == '\t' {
        acc + TAB_LENGTH - (acc % TAB_LENGTH)
    } else {
        acc + 1
    }
}
