use std::collections::HashMap;
use std::io::Write;

use failure::Error;
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use xrl::{Line, LineCache, Style, Update, ConfigChanges};

use super::client::Client;
use super::style::{reset_style, set_style};
use super::window::Window;
use super::cfg::ViewConfig;

#[derive(Debug, Default)]
pub struct Cursor {
    pub line: u64,
    pub column: u64,
}

pub struct View {
    cache: LineCache,
    cursor: Cursor,
    window: Window,
    file: Option<String>,
    client: Client,
    cfg: ViewConfig
}

impl View {
    pub fn new(client: Client, file: Option<String>) -> View {
        View {
            cache: LineCache::default(),
            cursor: Default::default(),
            window: Window::new(),
            cfg: ViewConfig::default(),
            client,
            file,
        }
    }

    pub fn update_cache(&mut self, update: Update) {
        info!("updating cache");
        self.cache.update(update)
    }

    pub fn set_cursor(&mut self, line: u64, column: u64) {
        self.cursor = Cursor { line, column };
        self.window.set_cursor(&self.cursor);
    }

    pub fn config_changed(&mut self, changes: ConfigChanges) {
        if let Some(tab_size) = changes.tab_size {
            self.cfg.tab_size = tab_size as u16;
        }
    }

    pub fn render<W: Write>(
        &mut self,
        w: &mut W,
        styles: &HashMap<u64, Style>,
    ) -> Result<(), Error> {
        self.update_window();
        self.render_lines(w, styles)?;
        self.render_cursor(w);
        Ok(())
    }

    pub fn resize(&mut self, height: u16) {
        self.window.resize(height);
        self.update_window();
        let top = self.cache.before() + self.window.start();
        let bottom = self.cache.after() + self.window.end();
        self.client.scroll(top, bottom);
    }

    pub fn insert(&mut self, c: char) {
        self.client.insert(c)
    }

    pub fn insert_newline(&mut self) {
        self.client.insert_newline()
    }

    pub fn insert_tab(&mut self) {
        self.client.insert_tab()
    }

    pub fn save(&mut self) {
        self.client.save(self.file.as_ref().unwrap())
    }

    pub fn back(&mut self) {
        self.client.backspace()
    }

    pub fn delete(&mut self) {
        self.client.delete()
    }

    pub fn page_down(&mut self) {
        self.client.page_down()
    }

    pub fn page_up(&mut self) {
        self.client.page_up()
    }

    pub fn move_left(&mut self) {
        self.client.left()
    }

    pub fn move_right(&mut self) {
        self.client.right()
    }

    pub fn move_up(&mut self) {
        self.client.up()
    }

    pub fn move_down(&mut self) {
        self.client.down()
    }

    pub fn toggle_line_numbers(&mut self) {
        self.cfg.display_gutter = !self.cfg.display_gutter;
    }

    fn update_window(&mut self) {
        if self.cursor.line < self.cache.before() {
            error!(
                "cursor is on line {} but there are {} invalid lines in cache.",
                self.cursor.line,
                self.cache.before()
            );
            return;
        }
        let cursor_line = self.cursor.line - self.cache.before();
        let nb_lines = self.cache.lines().len() as u64;
        self.cfg.gutter_size = (self.cache.before() + nb_lines + self.cache.after()).to_string().len() as u16;
        self.window.update(cursor_line, nb_lines);
    }

    fn get_click_location(&self, x: u64, y: u64) -> (u64, u64) {
        let lineno = x + self.cache.before() + self.window.start();
        if let Some(line) = self.cache.lines().get(x as usize) {
            if y < self.cfg.gutter_size as u64+1 {
                return (lineno, 0);
            }
            let mut text_len: u16 = 0;
            for (idx, c) in line.text.chars().enumerate() {
                let char_width = self.translate_char_width(text_len, c);
                text_len += char_width;
                if u64::from(text_len) >= y {
                    // If the character at idx is wider than one column,
                    // the click occurred within the character. Otherwise,
                    // the click occurred on the character at idx + 1
                    if char_width > 1 {
                        return (lineno as u64, (idx-self.cfg.gutter_size as usize) as u64);
                    } else {
                        return (lineno as u64, (idx-self.cfg.gutter_size as usize) as u64 + 1);
                    }
                }
            }
            return (lineno, line.text.len() as u64 + 1);
        } else {
            warn!("no line at index {} found in cache", x);
            return (x, y);
        }
    }

    fn click(&mut self, x: u64, y: u64) {
        let (line, column) = self.get_click_location(x, y);
        self.client.click(line, column);
    }

    fn drag(&mut self, x: u64, y: u64) {
        let (line, column) = self.get_click_location(x, y);
        self.client.drag(line, column);
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Char(c) => match c {
                    '\n' => self.insert_newline(),
                    '\t' => self.insert_tab(),
                    _ => self.insert(c),
                }
                Key::Ctrl(c) => match c {
                    'w' => self.save(),
                    'h' => self.back(),
                    _ => error!("un-handled input ctrl+{}", c),
                },
                Key::Backspace => self.back(),
                Key::Delete => self.delete(),
                Key::Left => self.client.left(),
                Key::Right => self.client.right(),
                Key::Up => self.client.up(),
                Key::Down => self.client.down(),
                Key::Home => self.client.home(),
                Key::End => self.client.end(),
                Key::PageUp => self.page_up(),
                Key::PageDown => self.page_down(),
                k => error!("un-handled key {:?}", k),
            },
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Press(press_event, y, x) => match press_event {
                    MouseButton::Left => self.click(u64::from(x) - 1, u64::from(y) - 1),
                    MouseButton::WheelUp => self.client.up(),
                    MouseButton::WheelDown => self.client.down(),
                    button => error!("un-handled button {:?}", button),
                },
                MouseEvent::Release(..) => {}
                MouseEvent::Hold(y, x) => self.drag(u64::from(x) - 1, u64::from(y) - 1),
            },
            ev => error!("un-handled event {:?}", ev),
        }
    }

    fn render_lines<W: Write>(&self, w: &mut W, styles: &HashMap<u64, Style>) -> Result<(), Error> {
        debug!("rendering lines");
        trace!("current cache\n{:?}", self.cache);

        // Get the lines that are within the displayed window
        let lines = self
            .cache
            .lines()
            .iter()
            .skip(self.window.start() as usize)
            .take(self.window.size() as usize);



        // Draw the valid lines within this range
        let mut line_strings = String::new();
        let mut line_no = self.cache.before()+self.window.start();
        for (line_index, line) in lines.enumerate() {
            line_strings.push_str(&self.render_line_str(line, Some(line_no), line_index, styles));
            line_no += 1;
        }

        // If the number of lines is less than window height
        // render empty lines to fill the view window.
        let line_count = self.cache.lines().len() as u16;
        let win_size = self.window.size();
        if win_size > line_count {
            for num in line_count..win_size {
                line_strings.push_str(&self.render_line_str(
                    &Line::default(),
                    None,
                    num as usize,
                    styles,
                ));
            }
        }
        w.write(line_strings.as_bytes())?;

        Ok(())
    }

    // Next tab stop, assuming 0-based indexing
    fn tab_width_at_position(&self, position: u16) -> u16 {
        self.cfg.tab_size - (position % self.cfg.tab_size)
    }

    fn render_line_str(&self, line: &Line, lineno: Option<u64>, line_index: usize, styles: &HashMap<u64, Style>) -> String {
        let text = self.escape_control_and_add_styles(styles, line);
        if let Some(line_no) = lineno {
            if self.cfg.display_gutter {
                format!("{}{}{}{}{}",
                    Goto(1, line_index as u16+1),
                    ClearLine,
                    (line_no+1).to_string(),
                    Goto(self.cfg.gutter_size+1, line_index as u16 + 1),
                    &text
                )
            } else {
                format!("{}{}{}", Goto(0, line_index as u16 + 1), ClearLine, &text)
            }
        } else {
            format!("{}{}{}", Goto(self.cfg.gutter_size+1, line_index as u16 + 1), ClearLine, &text)
        }
    }

    fn escape_control_and_add_styles(&self, styles: &HashMap<u64, Style>, line: &Line) -> String {
        let mut position: u16 = 0;
        let mut text = String::with_capacity(line.text.capacity());
        for c in line.text.chars() {
            match c {
                '\x00'...'\x08' | '\x0a'...'\x1f' | '\x7f' => {
                    // Render in caret notation, i.e. '\x02' is rendered as '^B'
                    text.push('^');
                    text.push((c as u8 ^ 0x40u8) as char);
                    position += 2;
                },
                '\t' => {
                    let tab_width = self.tab_width_at_position(position);
                    text.push_str(&" ".repeat(tab_width as usize));
                    position += tab_width;
                },
                _ => {
                    text.push(c);
                    position += 1;
                },
            }
        }
        if line.styles.is_empty() {
            return text;
        }
        let mut style_sequences = self.get_style_sequences(styles, line);
        for style in style_sequences.drain(..) {
            trace!("inserting style: {:?}", style);
            if style.0 >= text.len() {
                text.push_str(&style.1);
            } else {
                text.insert_str(style.0, &style.1);
            }
        }
        trace!("styled line: {:?}", text);
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
                            style, e
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

    fn render_cursor<W: Write>(&self, w: &mut W) {
        info!("rendering cursor");
        if self.cache.is_empty() {
            info!("cache is empty, rendering cursor at the top left corner");
            if let Err(e) = write!(w, "{}", Goto(1, 1)) {
                error!("failed to render cursor: {}", e);
            }
            return;
        }

        if self.cursor.line < self.cache.before() {
            error!(
                "the cursor is on line {} which is marked invalid in the cache",
                self.cursor.line
            );
            return;
        }
        // Get the line that has the cursor
        let line_idx = self.cursor.line - self.cache.before();
        let line = match self.cache.lines().get(line_idx as usize) {
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
        // control characters and tabs. We assume control characters (0x00-0x1f, excluding 0x09 ==
        // tab) are rendered in caret notation and are thus two columns wide. Tabs are
        // variable-width, rounding up to the next tab stop. All other characters are assumed to be
        // one column wide.
        let column: u16 = line
            .text
            .chars()
            .take(self.cursor.column as usize)
            .fold(0, |acc, c| { acc + self.translate_char_width(acc, c) });

        // Draw the cursor
        let cursor_pos = Goto(self.cfg.gutter_size + column + 1, line_pos as u16 + 1);
        if let Err(e) = write!(w, "{}", cursor_pos) {
            error!("failed to render cursor: {}", e);
        }
        info!("Cursor rendered at ({}, {})", line_pos, column);
    }

    fn translate_char_width(&self, position: u16, c: char) -> u16 {
        match c {
            // Caret notation means non-tab control characters are two columns wide
            '\x00'...'\x08' | '\x0a'...'\x1f' | '\x7f' => 2,
            '\t' => self.tab_width_at_position(position),
            _ => 1
        }
    }
}
