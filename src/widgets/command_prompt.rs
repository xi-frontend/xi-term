use std::io::Error;
use std::io::Write;
use termion::event::{Event, Key};

use core::Command;
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

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
        // TODO: Clean this up.
        let cmd = match &self.chars[..] {
            "s" | "save" => Some(Command::Save(None)),
            "q" | "quit" => Some(Command::Quit),
            command => {
                match &command[0..1] {
                    "t" => {
                        if &command[1..2] == " " {
                            Some(Command::SetTheme(command[2..].to_owned()))
                        } else if &command[0..6] == "theme " {
                            Some(Command::SetTheme(command[6..].to_owned()))
                        } else {
                            error!("Received invalid theme: {:?}", &command[6..]);
                            None
                        }
                    },
                    "o" => {
                        if &command[1..2] == " " {
                            if command[2..].len() > 0 {
                                Some(Command::Open(Some(command[2..].to_owned())))
                            } else {
                                Some(Command::Open(None))
                            }
                        } else if &command[0..5] == "open " {
                            if command[5..].len() > 0 {
                                Some(Command::Open(Some(command[5..].to_owned())))
                            } else {
                                Some(Command::Open(None))
                            }
                        } else {
                            error!("Received invalid theme: {:?}", &command[5..]);
                            None
                        }
                    },
                    _ => {
                        error!("Received invalid command: {:?}", command);
                        Some(Command::Invalid(command.to_owned()))
                    }
                }
            }
        };
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
