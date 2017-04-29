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
use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

use serde_json;

use core::Core;
use view::View;

pub struct Screen {
    pub stdout: MouseTerminal<AlternateScreen<RawTerminal<std::io::Stdout>>>,
    pub size: (u16, u16),
}

impl Screen {
    pub fn new() -> Screen {
        let stdout = MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
        Screen {
            size: termion::terminal_size().unwrap(),
            stdout: stdout,
        }
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
        write!(self.stdout, "{}", clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn update(&mut self, core: &mut Core) {
        // TODO(#27): check if terminal size changed. If so, send a `render_line` command to the
        // backend, and a `scroll` command for future updates.
        if let Ok(msg) = core.update_rx.try_recv() {
            let msg_list = msg.as_array().unwrap();
            let (method, params) = (msg_list[0].as_str().unwrap(),
                                    msg_list[1].as_object().unwrap());
            match method {
                "update" => {
                    let update = serde_json::from_value(params.get("update").unwrap().clone())
                        .unwrap();
                    core.update(&update);

                    let view = core.view();
                    self.draw(&view);
                }
                "scroll_to" => {
                    let (col, line) = (params.get("col").unwrap().as_u64().unwrap(),
                                       params.get("line").unwrap().as_u64().unwrap());
                    self.scroll(col, line);
                }
                "set_style" => {
                    // TODO:(#26): ???
                }
                _ => {
                    info!("Unknown request from backend {:?}", method);
                }
            }
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
