//! Command prompt for xi-term. currently this is
//! heavily inspired by vim and is just designed to
//! get a simple base to work off of.

use std::io::Error;
use std::io::Write;
use termion::event::{Event, Key};

use crate::core::{Command, ParseCommandError, FromPrompt, FindConfig, Keymap};
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandPromptMode {
    // Parse commands from user-input
    Command,
    // Switch directly to search-mode
    Find,
}

#[derive(Debug)]
pub struct CommandPrompt {
    mode: CommandPromptMode,
    dex: usize,
    chars: String,
    keybindings: Keymap
}

impl CommandPrompt {
    pub fn new(mode: CommandPromptMode, keybindings: Keymap) -> CommandPrompt {
        CommandPrompt{mode, dex: 0, chars: Default::default(), keybindings}
    }

    /// Process a terminal event for the command prompt.
    pub fn handle_input(&mut self, input: &Event) -> Result<Option<Command>, ParseCommandError> {
        match input {
            Event::Key(Key::Char('\n')) => self.finalize(),
            Event::Key(Key::Backspace) | Event::Key(Key::Ctrl('h')) => Ok(self.back()),
            Event::Key(Key::Delete) => Ok(self.delete()),
            Event::Key(Key::Left) => Ok(self.left()),
            Event::Key(Key::Right) => Ok(self.right()),
            Event::Key(Key::Char(chr)) => Ok(self.new_key(*chr)),
            _ => Ok(None),
        }
    }

    fn left(&mut self) -> Option<Command> {
        if self.dex > 0 {
            self.dex -= 1;
        }
        None
    }

    fn right(&mut self) -> Option<Command> {
        if self.dex < self.chars.len() {
            self.dex += 1;
        }
        None
    }

    fn delete(&mut self) -> Option<Command> {
        if self.dex < self.chars.len() {
            self.chars.remove(self.dex);
        }
        None
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

    /// Gets called when any character is pressed.
    fn new_key(&mut self, chr: char) -> Option<Command> {
        self.chars.insert(self.dex, chr);
        self.dex += 1;
        None
    }

    /// Gets called when return is pressed,
    fn finalize(&mut self) -> Result<Option<Command>, ParseCommandError> {
        match self.mode {
            CommandPromptMode::Find => Ok(Some(FindConfig::from_prompt(&self.chars)?)),
            CommandPromptMode::Command => Ok(Some(Command::from_prompt(&self.chars)?)),
        }
        
    }

    fn render_suggestions<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        if self.chars.is_empty() {
            return Ok(())
        }

        let vals : Vec<_> = self.keybindings.values().filter(|x| x.name.starts_with(&self.chars)).take(4).collect();
        for (idx, val) in vals.iter().enumerate() {
            if let Err(err) = write!(
                w,
                "{}{}-> {}   [{}]",
                Goto(1, row - 1 - idx as u16),
                ClearLine,
                val.name,
                val.keys,
            ) {
                error!("failed to render status bar: {:?}", err);
                // TODO: Return error
            }
        }
        Ok(())
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        let mode_indicator; 

        match self.mode {
            CommandPromptMode::Find => {
                mode_indicator = "find";

                // Write a line explaining the search above the searchbar
                if let Err(err) = write!(
                    w,
                    "{}{}Prefix your search with r, c and/or w \
                    to configure search to be (r)egex, (c)ase_sensitive, (w)hole_words. \
                    All false by default. Example: \"cw Needle\"",
                    Goto(1, row - 1),
                    ClearLine,
                ) {
                    error!("failed to render status bar: {:?}", err);
                }
            }   
            CommandPromptMode::Command => {
                mode_indicator = "";
                self.render_suggestions(w, row)?;
            },
        };

        let cursor_start = (self.dex + 2 + mode_indicator.len()) as u16;

        if let Err(err) = write!(
            w,
            "{}{}{}:{}{}",
            Goto(1, row),
            ClearLine,
            mode_indicator,
            self.chars,
            Goto(cursor_start, row)
        ) {
            error!("failed to render status bar: {:?}", err);
        }
        Ok(())
    }
}
