//! Command prompt for xi-term. currently this is
//! heavily inspired by vim and is just designed to
//! get a simple base to work off of.

use std::io::Error;
use std::io::Write;
use termion::event::{Event, Key};

use crate::core::{Command, ParseCommandError, FromPrompt, FindConfig, ParserMap};
use termion::clear::CurrentLine as ClearLine;
use termion::cursor::Goto;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandPromptMode {
    /// Do not display Prompt
    Inactive,
    /// Parse commands from user-input
    Command,
    /// Switch directly to search-mode
    Find,
}

pub struct CommandPrompt {
    mode: CommandPromptMode,
    dex: usize,
    chars: String,
    prompt_texts: Vec<String>,
    parser_map: ParserMap,
}

impl CommandPrompt {
    pub fn new(mode: CommandPromptMode, parser_map: ParserMap) -> CommandPrompt {
        let mut prompt_texts = Vec::new();
        for (key, parser) in &parser_map {
            let keybinding = parser.keybinding.clone().unwrap_or(String::new());
            if parser.subcommands.is_empty() {
                prompt_texts.push(format!("{}   <{}>", key, &keybinding));
            } else {
                for subcommand in &parser.subcommands {
                    prompt_texts.push(format!("{} {}   <{}>", key, subcommand, &keybinding));
                }
            }
            // prompt_texts.push(format!("{} {}    [{}]", key, );
        }
        CommandPrompt{mode, dex: 0, chars: Default::default(), prompt_texts, parser_map}
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
        let res = match self.mode {
            CommandPromptMode::Find => Ok(Some(FindConfig::from_prompt(Some(&self.chars))?)),
            CommandPromptMode::Command => {
                // Split first word off, search for it in the map and hand the rest to the from_prompt-command
                let mut splitvec = self.chars.splitn(2, ' ');
                let cmd_name = splitvec.next().unwrap(); // Should not panic
                let add_args = splitvec.next();
                if let Some(parser) = self.parser_map.get::<str>(&cmd_name) {
                    Ok(Some((parser.from_prompt)(add_args)?))
                } else {
                    Err(ParseCommandError::UnexpectedArgument)
                }
            },
            // Shouldn't happen
            CommandPromptMode::Inactive => Err(ParseCommandError::UnexpectedArgument)
        };
        // Prompt was finalized. Close it now. 
        self.mode = CommandPromptMode::Inactive;
        res
    }

    fn render_suggestions<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        if self.chars.is_empty() {
            return Ok(())
        }

        let vals : Vec<_> = self.prompt_texts.iter().filter(|x| x.starts_with(&self.chars)).take(4).collect();
        for (idx, val) in vals.iter().enumerate() {
            if let Err(err) = write!(
                w,
                "{}{}-> {}",
                Goto(1, row - 1 - idx as u16),
                ClearLine,
                val,
            ) {
                error!("failed to render status bar: {:?}", err);
                // TODO: Return error
            }
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.mode != CommandPromptMode::Inactive
    }

    pub fn set_mode(&mut self, mode: CommandPromptMode) {
        self.mode = mode;
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        let mode_indicator; 

        match self.mode {
            CommandPromptMode::Inactive => {return Ok(());}
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
