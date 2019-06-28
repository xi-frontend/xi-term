//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
use xrl::ViewId;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::KeymapEntry;
use crate::widgets::CommandPromptMode;

pub trait FromPrompt {
    fn from_prompt(vals: &str) -> Result<Command, ParseCommandError>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum RelativeMoveDistance {
    /// Move only one character
    characters,
    /// Move a line
    lines,
    /// Move to new word
    words,
    /// Move to end of word
    word_ends,
    /// Move to new subword
    subwords,
    /// Move to end of subword
    subword_ends,
    /// Move a page
    pages,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RelativeMove {
    pub by: RelativeMoveDistance,
    pub forward: bool,
    #[serde(default)]
    pub extend: bool
}

impl FromPrompt for RelativeMove {
    fn from_prompt(args: &str) -> Result<Command, ParseCommandError> {
        let vals : Vec<&str> = args.split(' ').collect();
        if vals.is_empty() {
            return Err(ParseCommandError::ExpectedArgument{cmd: "move".to_string()});
        }

        if vals.len() > 2 {
            return Err(ParseCommandError::TooManyArguments{cmd: "move".to_string(), expected: 2, found: vals.len()});
        }

        let extend = vals.len() == 2;
        match vals[0] {
            "d" | "down" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::lines, 
                                            forward: true, 
                                            extend
                                            }
                               )),
            "u" | "up" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::lines, 
                                            forward: false, 
                                            extend
                                            }
                               )),
            "r" | "right" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::characters, 
                                            forward: true, 
                                            extend
                                            }
                               )),
            "l" | "left" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::characters, 
                                            forward: false, 
                                            extend
                                            }
                               )),
            "pd" | "page-down" => Ok(Command::RelativeMove(
                                        RelativeMove{
                                                    by: RelativeMoveDistance::pages, 
                                                    forward: true, 
                                                    extend
                                                    }
                                       )),
            "pu" | "page-up" => Ok(Command::RelativeMove(
                                        RelativeMove{
                                                    by: RelativeMoveDistance::pages, 
                                                    forward: false, 
                                                    extend
                                                    }
                                       )),
            command => Err(ParseCommandError::UnknownCommand(command.into()))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum AbsoluteMovePoint {
    /// Beginning of file
    bof,
    /// End of file
    eof,
    /// Beginning of line
    bol,
    /// End of line
    eol,
    /// Enclosing brackets
    brackets,
    /// Line number
    line(u64)
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AbsoluteMove {
    pub to: AbsoluteMovePoint,
    #[serde(default)]
    pub extend: bool
}

impl FromPrompt for AbsoluteMove {
    fn from_prompt(args: &str) -> Result<Command, ParseCommandError> {
        let vals : Vec<&str> = args.split(' ').collect();
        if vals.is_empty() {
            return Err(ParseCommandError::ExpectedArgument{cmd: "move_to".to_string()});
        }

        if vals.len() > 2 {
            return Err(ParseCommandError::TooManyArguments{cmd: "move_to".to_string(), expected: 2, found: vals.len()});
        }

        let extend = vals.len() == 2;
        match vals[0] {
            "bof" | "beginning-of-file" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::bof,
                                                                extend
                                                                }
                                                   )),
            "eof" | "end-of-file" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::eof,
                                                                extend
                                                                }
                                                   )),
            "bol" | "beginning-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::bol,
                                                                extend
                                                                }
                                                   )),
            "eol" | "end-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::eol,
                                                                extend
                                                                }
                                                   )),
            command => Err(ParseCommandError::UnknownCommand(command.into()))
        }

    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ExpandLinesDirection {
    pub forward: bool
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    /// Close the CommandPrompt.
    Cancel,
    /// Quit editor.
    Quit,
    /// Save the current file buffer.
    Save(Option<ViewId>),
    /// Backspace
    Back,
    /// Delete
    Delete,
    /// Open A new file.
    Open(Option<String>),
    /// Cycle to the next View.
    NextBuffer,
    /// Cycle to the previous buffer.
    PrevBuffer,
    /// Relative move like line up/down, page up/down, left, right, word left, ..
    RelativeMove(RelativeMove),
    /// Relative move like line ending/beginning, file ending/beginning, line-number, ...
    AbsoluteMove(AbsoluteMove),
    /// Change current color theme
    SetTheme(String),
    /// Toggle displaying line numbers.
    ToggleLineNumbers,
    /// Open prompt for user-input
    OpenPrompt(CommandPromptMode),
    /// Insert a character
    Insert(char),
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,
    /// Find the given string
    Find(String),
    /// Find next occurence of active search
    FindNext,
    /// Find previous occurence of active search
    FindPrev,
    /// Find word and set another cursor there
    FindUnderExpand,
    /// Set a new cursor below or above current position
    CursorExpandLines(ExpandLinesDirection),
    /// Copy the current selection
    CopySelection,
    /// Paste previously copied or cut text
    Paste,
    /// Copy the current selection
    CutSelection,
    /// Close the current view
    CloseCurrentView,
    /// Select all text in the current view
    SelectAll,
}

#[derive(Debug)]
pub enum ParseCommandError {
    /// Didnt expect a command to take an argument.
    UnexpectedArgument,
    /// The given command expected an argument.
    ExpectedArgument {
        cmd: String,
        // expected: usize,
        // found: usize,
    },
    /// The given command was given to many arguments.
    TooManyArguments {
        cmd: String,
        expected: usize,
        found: usize,
    },
    /// Invalid input was received.
    UnknownCommand(String),
}

impl Command {

    pub fn from_keymap_entry(val: KeymapEntry) -> Result<Command, ParseCommandError> {
        match val.command.as_ref() {
            "select_all" => Ok(Command::SelectAll),
            "close" => Ok(Command::CloseCurrentView),
            "copy" => Ok(Command::CopySelection),
            "cut" => Ok(Command::CutSelection),
            "paste" => Ok(Command::Paste),
            "fue" | "find_under_expand" => Ok(Command::FindUnderExpand),
            "fn" | "find_next" => Ok(Command::FindNext),
            "fp" | "find_prev" => Ok(Command::FindPrev),
            "hide_overlay" => Ok(Command::Cancel),
            "s" | "save" => Ok(Command::Save(None)),
            "q" | "quit" | "exit" => Ok(Command::Quit),
            "b" | "back" | "left_delete" => Ok(Command::Back),
            "d" | "delete" | "right_delete" => Ok(Command::Delete),
            "bn" | "next-buffer" | "next_view" => Ok(Command::NextBuffer),
            "bp" | "prev-buffer" | "prev_view" => Ok(Command::PrevBuffer),
            "undo" => Ok(Command::Undo),
            "redo" => Ok(Command::Redo),
            "ln" | "line-numbers" => Ok(Command::ToggleLineNumbers),
            "op" | "open-prompt" => Ok(Command::OpenPrompt(CommandPromptMode::Command)),
            "show_overlay" => {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "show_overlay".to_string()})?;
                match args.get("overlay") {
                    None => Err(ParseCommandError::UnexpectedArgument),
                    Some(value) => match value {
                                        // We should catch "command_palette" here instead, but because of a bug in termion
                                        // we can't parse ctrl+shift+p...
                                        // Later on we might introduce another prompt mode for "goto" as well.
                                        Value::String(x) if x == "goto" => Ok(Command::OpenPrompt(CommandPromptMode::Command)),
                                        _ => Err(ParseCommandError::UnexpectedArgument),
                                   }
                }
            }

            "show_panel" => {
                error!("+++++++++++++++ A1");
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "show_panel".to_string()})?;
                match args.get("panel") {
                    None => Err(ParseCommandError::UnexpectedArgument),
                    Some(value) => { error!("+++++++++++++++ A2: {}", value);; match value {
                                        Value::String(x) if x == "find" => Ok(Command::OpenPrompt(CommandPromptMode::Find)),
                                        _ => Err(ParseCommandError::UnexpectedArgument),
                                   }}
                }
            }


            "move"    => {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move".to_string()})?;
                let cmd : RelativeMove = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::RelativeMove(cmd))
            },
            "move_to" => {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move_to".to_string()})?;
                let cmd : AbsoluteMove = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::AbsoluteMove(cmd))
            },
            "select_lines" => {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "select_lines".to_string()})?;
                let cmd : ExpandLinesDirection = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::CursorExpandLines(cmd))
            },
            command => Err(ParseCommandError::UnknownCommand(command.into())),
        }
    }
}

impl FromPrompt for Command {
    fn from_prompt(input: &str) -> Result<Command, ParseCommandError> {
        let mut parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts.remove(0);

        // If we have prompt-arguments, we parse them directly to a command instead of going via json
        let args = parts.get(0);
        match cmd.as_ref() {
            // First, catch some prompt-specific commands (usually those with arguments),
            // which need different parsing than whats coming from the keymap-file
            "move"    => {
                let arg = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move".to_string()})?;
                RelativeMove::from_prompt(arg)
            },
            "move_to" => {
                let arg = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move".to_string()})?;
                AbsoluteMove::from_prompt(arg)
            },
            "t" | "theme" => {
                let theme = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "theme".to_string()})?;
                Ok(Command::SetTheme(theme.to_string()))
            },
            "o" | "open" => {
                // Don't split given arguments by space, as filenames can have spaces in them as well!
                let filename = match args {
                    Some(name) => {
                        // We take the value given from the prompt and run it through shellexpand,
                        // to translate to a real path (e.g. "~/.bashrc" doesn't work without this)
                        let expanded_name = shellexpand::full(name)
                                               .map_err(|_| ParseCommandError::UnknownCommand(name.to_string()))?;
                        Some(expanded_name.to_string())
                    },

                    // If no args where given we open with "None", which is ok, too.
                    None => None,
                };
                Ok(Command::Open(filename))
            }

            "f" | "find" => {
                let needle = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "find".to_string()})?;
                Ok(Command::Find(needle.to_string()))
            },

            // The stuff we don't handle here, we pass on to the default parsing function
            // Since there is no way to know the shape of "args", we drop all 
            // potentially given prompt-args for this command here.
            command => Command::from_keymap_entry(KeymapEntry{keys: Vec::new(), 
                                                  command: command.to_string(), 
                                                  args: None, 
                                                  context: None})
        }
    }
}
