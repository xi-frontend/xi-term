use std;
use std::cmp;
use std::io::stdout;
use std::io::Write;
use std::thread;
use std::time;

use termion;
use termion::clear;
use termion::cursor;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

use core::Core;
use update::Update;
use view::View;

pub struct Screen {
    pub stdout: MouseTerminal<RawTerminal<std::io::Stdout>>,
    pub size: (u16, u16),
}

impl Screen {
    pub fn new() -> Screen {
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        write!(stdout, "{}", clear::All).unwrap();
        stdout.flush().unwrap();
        Screen {
            size: termion::terminal_size().unwrap(),
            stdout: stdout,
        }
    }

    // TODO: handle lines that are longer than terminal width.
    // Should we wrap them or truncate them?
    pub fn redraw(&mut self, _update: &Update) {
        unimplemented!();
        /*
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();

        let nb_lines = cmp::min(update.lines.len(), self.size.1 as usize);
        if nb_lines > 0 {
            for line in update.lines.iter().take(nb_lines - 1) {
                write!(self.stdout, "{}", cursor::Left(self.size.0)).unwrap();

                if let Some(selection) = line.selection {
                    let start = selection.0 as usize;
                    let end = selection.1 as usize;
                    let mut str_before = String::new();
                    for c in line.text.chars().take(start) {
                        str_before.push(c);
                    }
                    let mut str_selection = String::new();
                    for c in line.text.chars().skip(start).take(end) {
                        str_selection.push(c);
                    }
                    let mut str_after = String::new();
                    for c in line.text.chars().skip(end) {
                        str_after.push(c);
                    }
                    write!(self.stdout, "{}{}{}{}{}{}",
                           termion::style::Reset,
                           str_before,
                           termion::color::Bg(color::Red),
                           str_selection,
                           termion::style::Reset,
                           str_after).unwrap();
                } else {
                    self.stdout.write_all(line.text.as_bytes()).unwrap();
                }
            }

            // If the last line has a trailing \n, we need to remove it
            let mut last_line = update.lines[nb_lines - 1].text.clone();
            match last_line.pop() {
                Some('\n') | None => {
                },
                Some(c) => {
                    last_line.push(c);
                }
            }
            write!(self.stdout, "{}", cursor::Left(self.size.0)).unwrap();
            self.stdout.write_all(last_line.as_bytes()).unwrap();
        }

        let cursor_line_idx = update.scroll_to.0 - update.first_line;
        let cursor_line = update.lines[cursor_line_idx as usize].text.clone();
        let mut cols =  0;
        for c in cursor_line.chars().take(update.scroll_to.1 as usize) {
            if c == '\t' {
                cols += 4;
            } else {
                cols += 1;
            }
        }
        self.scroll(cols + 1, cursor_line_idx + 1);
        */
    }

    pub fn draw(&mut self, view: &View) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();

        let range = 0..(cmp::min(view.lines.len(), self.size.1 as usize));
        for (lineno, line) in range.zip(view.lines.iter()) {
            write!(self.stdout, "{}", line.text.clone().unwrap()).unwrap();
            self.scroll(0, lineno as u64);
        }
    }

    pub fn scroll(&mut self, col: u64, line: u64) {
        write!(self.stdout, "{}", cursor::Goto((col + 1) as u16, (line + 1) as u16)).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn init(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn update(&mut self, core: &mut Core) {
        // TODO: check if terminal size changed. If so, send a `render_line` command to the backend,
        // and a `scroll` command for future updates.
        if let Ok(msg) = core.update_rx.try_recv() {
            let msg_list = msg.as_array().unwrap();
            let (method, params) = (msg_list[0].as_str().unwrap(), msg_list[1].as_object().unwrap());
            if method == "update" {
                let update = Update::from_value(params.get("update").unwrap());
                core.update(&update);

                let view = core.view();
                self.draw(&view);
            } else if method == "scroll_to" {
                let (col, line) = (params.get("col").unwrap().as_u64().unwrap(), params.get("line").unwrap().as_u64().unwrap());
                self.scroll(col, line);
            } else if method == "set_style" {
                // TODO: ???
            } else {
                info!("Unknown request from backend {:?}", method);
            }
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
