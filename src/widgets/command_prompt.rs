use std::io::Error;
use std::io::Write;
use termion::event::{Event, Key};

use core::Command;
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

use std::str::FromStr;

/// Command prompt for xi-term.
/// currently this is heavily inspired by vim
/// and is just disigned to get a simple base to work off of.
#[derive(Debug, Default)]
pub struct CommandPrompt {
    dex: usize,
    chars: String,
}

impl CommandPrompt {
    /// Process a terminal event for the command prompt.
    pub fn handle_input(&mut self, input: &Event) -> Option<Command> {
        // TODO: not ignore arrow keys
        match input {
            Event::Key(Key::Char('\n')) => self.finalize(),
            Event::Key(Key::Backspace) => self.back(),
            Event::Key(Key::Delete) => {
                if self.dex < self.chars.len() {
                    self.chars.remove(self.dex);
                }
                None
            }
            Event::Key(Key::Left) => {
                if self.dex > 0 {
                    self.dex -= 1;
                }
                None
            },
            Event::Key(Key::Right) => {
                if self.dex+1 < self.chars.len() {
                    self.dex += 1;
                }
                None
            },
            Event::Key(Key::Char(chr)) => self.new_key(*chr),
            _ => None,
        }
    }

    fn back(&mut self) -> Option<Command> {
        if !self.chars.is_empty() {
            self.dex -= 1;
            self.chars.remove(self.dex);
            None
        } else {
            Some(Command::Cancel)
        }
    }

    fn new_key(&mut self, chr: char) -> Option<Command> {
        self.chars.insert(self.dex, chr);
        self.dex += 1;
        None
    }

    /// Gets called when return is pressed,
    fn finalize(&mut self) -> Option<Command> {
        match FromStr::from_str(&self.chars) {
            Ok(cmd) => Some(cmd),
            Err(err) => {
                error!("Failed to parse Command: {:?}", err);
                None
            }
        }
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        info!("Rendering Status bar at this Row: {}", row);
        if let Err(err) = write!(w, "{}{}:{}{}", Goto(1, row), ClearLine, self.chars, Goto(self.dex as u16+2, row)) {
            error!("faile to render status bar: {:?}", err);
        }
        Ok(())
    }
}
