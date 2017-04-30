use std::{self, cmp, thread, time};
use std::io::{stdout, Write};

use serde_json;

use termion::{self, clear, cursor, style};
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use core::Core;
use errors::*;
use view::View;

pub struct Screen {
    pub stdout: MouseTerminal<AlternateScreen<RawTerminal<std::io::Stdout>>>,
    pub size: (u16, u16),
}

impl Screen {
    pub fn new() -> Result<Screen> {
        let stdout = MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode()?));
        Ok(Screen {
               size: termion::terminal_size()?,
               stdout: stdout,
           })
    }

    pub fn draw(&mut self, view: &View) -> Result<()> {
        write!(self.stdout, "{}{}", clear::All, cursor::Up(self.size.1))
            .chain_err(|| ErrorKind::DisplayError)?;

        let range = 0..(cmp::min(view.lines.len(), self.size.1 as usize));
        for (lineno, line) in range.zip(view.lines.iter()) {
            if line.is_valid {
                let text = line.render()?;
                write!(self.stdout, "{}{}{}", cursor::Goto(1, 1 + lineno as u16), text, cursor::Hide)
                    .chain_err(|| ErrorKind::DisplayError)?;
            }
        }
        self.stdout
            .flush()
            .chain_err(|| ErrorKind::DisplayError)?;
        Ok(())
    }

    pub fn scroll_to(&mut self, _col: u64, _line: u64) {
        // We draw "fake" cursor(s) while rendering the lines. so there's nothing to do here.
        // However, on the long term, we do want to set the cursor correctly, so that it can blink.
        // One problem is that xi-core returns gives the position of the cursor as a string index.
        // But we have some characters that are more or less large: a tab can be multiple spaces
        // for example. So we need some logic to know exactly where the cursor should be set.
    }

    pub fn init(&mut self) -> Result<()> {
        write!(self.stdout, "{}{}", clear::All, cursor::Up(self.size.1))
            .chain_err(|| ErrorKind::DisplayError)?;
        self.stdout
            .flush()
            .chain_err(|| ErrorKind::DisplayError)?;
        Ok(())
    }

    pub fn update(&mut self, core: &mut Core) -> Result<()> {
        // TODO(#27): check if terminal size changed. If so, send a `render_line` command to the
        // backend, and a `scroll` command for future updates.
        if let Ok(msg) = core.update_rx.try_recv() {
            let msg_list = msg.as_array().unwrap();
            let (method, params) = (msg_list[0].as_str().unwrap(),
                                    msg_list[1].as_object().unwrap());
            match method {
                "update" => {
                    let update = serde_json::from_value(params.get("update").unwrap().clone())?;
                    core.update(&update)?;
                    let view = core.get_view().ok_or(ErrorKind::DisplayError)?;
                    self.draw(view)?;
                }
                "scroll_to" => {
                    let (col, line) = (params.get("col").unwrap().as_u64().unwrap(),
                                       params.get("line").unwrap().as_u64().unwrap());
                    self.scroll_to(col, line);
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
        Ok(())
    }
}
