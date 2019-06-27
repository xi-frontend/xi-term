//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
use xrl::ViewId;

use serde::{Deserialize, Serialize};

use crate::core::KeymapEntry;

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
    // Relative move like line up/down, page up/down, left, right, word left, ..
    RelativeMove(RelativeMove),
    // Relative move like line ending/beginning, file ending/beginning, line-number, ...
    AbsoluteMove(AbsoluteMove),

    SetTheme(String),
    /// Toggle displaying line numbers.
    ToggleLineNumbers,
    /// Open prompt for user-input
    OpenPrompt,
    /// Insert a character
    Insert(char),
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,
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
    CloseCurrentView
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
            "close" => Ok(Command::CloseCurrentView),
            "copy" => Ok(Command::CopySelection),
            "cut" => Ok(Command::CutSelection),
            "paste" => Ok(Command::Paste),
            "fue" | "find_under_expand" => Ok(Command::FindUnderExpand),
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
            "op" | "open-prompt" | "show_overlay" => Ok(Command::OpenPrompt),
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

        // If no arguments are given, we can pass it along to the main parsing function
        if parts.is_empty() {
            return Command::from_keymap_entry(KeymapEntry{keys: Vec::new(), 
                                                          command: cmd.to_string(), 
                                                          args: None, 
                                                          context: None});
        }

        // If we have prompt-arguments, we parse them directly to a command instead of going via json
        let args = parts.remove(0);
        match cmd.as_ref() {
            "move"    => RelativeMove::from_prompt(args),
            "move_to" => AbsoluteMove::from_prompt(args),
            "t" | "theme" => {
                if args.is_empty() {
                    Err(ParseCommandError::ExpectedArgument {
                        cmd: "theme".into()
                    })
                } else {
                    Ok(Command::SetTheme(args.to_owned()))
                }
            }
            "o" | "open" => {
                let parts: Vec<&str> = args.split(' ').collect();
                if parts.is_empty() {
                    Ok(Command::Open(None))
                } else if parts.len() > 1 {
                    Err(ParseCommandError::UnexpectedArgument)
                } else {
                    let file = shellexpand::full(parts[0]).map_err(|_| ParseCommandError::UnknownCommand(parts[0].to_string()))?;
                    Ok(Command::Open(Some(file.to_string())))
                }
            }

            command => Err(ParseCommandError::UnknownCommand(command.into())),
        }
    }
}
