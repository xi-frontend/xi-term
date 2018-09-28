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
    chars: String,
}

impl CommandPrompt {
    /// Process a terminal event for the command prompt.
    pub fn handle_input(&mut self, input: &Event) -> Option<Command> {
        // TODO: not ignore arrow keys
        match input {
            Event::Key(Key::Char('\n')) => self.finalize(),
            Event::Key(Key::Backspace) => self.back(),
            Event::Key(Key::Char(chr)) => self.new_key(*chr),
            _ => None,
        }
    }

    fn back(&mut self) -> Option<Command> {
        if self.chars.is_empty() {
            self.chars.pop();
            None
        } else {
            Some(Command::Cancel)
        }
    }

    fn new_key(&mut self, chr: char) -> Option<Command> {
        self.chars.push(chr);
        None
    }

    /// Gets called when return is pressed,
    fn finalize(&mut self) -> Option<Command> {
        let cmd = FromStr::from_str(&self.chars).ok();
        self.chars = String::new();
        cmd
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        info!("Rendering Status bar at this Row: {}", row);
        if let Err(err) = write!(w, "{}{}:{}", Goto(1, row), ClearLine, self.chars) {
            error!("faile to render status bar: {:?}", err);
        }
        Ok(())
    }
}
