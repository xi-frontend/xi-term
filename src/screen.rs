use std;
use std::io::stdout;
use std::io::Write;
use std::thread;
use std::time;

use serde_json;

use termion;
use termion::clear;
use termion::cursor;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use core::Core;
use style::Style;
use errors::*;

pub struct Screen {
    pub stdout: MouseTerminal<AlternateScreen<RawTerminal<std::io::Stdout>>>,
    pub size: (u16, u16),
    update_scheduled: bool
}

impl Screen {
    pub fn new() -> Result<Screen> {
        let stdout = MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode()?));
        Ok(Screen {
            size: (0, 0),
            stdout: stdout,
            update_scheduled: false,
        })
    }

    pub fn schedule_update(&mut self) {
        self.update_scheduled = true;
    }

    /// Update the terminal size and return `true` if the height changed, and false otherwise.
    pub fn resize(&mut self) -> Result<Option<(u16, u16)>> {
        let new_size = termion::terminal_size().chain_err(|| ErrorKind::TerminalSizeError)?;
        if self.size.1 == new_size.1 {
            Ok(None)
        } else {
            self.size = new_size;
            Ok(Some(self.size))
        }
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
        if let Ok(msg) = core.update_rx.try_recv() {
            let msg_list = msg.as_array().unwrap();
            let (method, params) = (msg_list[0].as_str().unwrap(),
                                    msg_list[1].as_object().unwrap());
            match method {
                "update" => {
                    let update = serde_json::from_value(params.get("update").unwrap().clone())?;
                    core.update(&update)?;
                    self.schedule_update();
                }
                "scroll_to" => {
                    // Deserialize the cursor position, and let the core update the view.
                    let coord = (params.get("line").unwrap().as_u64().unwrap(),
                                 params.get("col").unwrap().as_u64().unwrap());
                    core.scroll_to(coord)?;
                    self.schedule_update();
                }
                "set_style" => {
                    let style: Style = serde_json::from_value(params.get("set_style").unwrap().clone())?;
                    core.get_view_mut()
                        .ok_or_else(|| {
                            error!("No view found");
                            ErrorKind::DisplayError
                        })?
                        .set_style(style);
                    self.schedule_update();
                }
                _ => {
                    info!("Unknown request from backend {:?}", method);
                }
            }
        }
        if self.update_scheduled {
            self.update_scheduled = false;
            core.get_view_mut()
                .ok_or_else(|| {
                    error!("No view found");
                    ErrorKind::DisplayError
                })?
                .render(&mut self.stdout)?
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
        Ok(())
    }

}
