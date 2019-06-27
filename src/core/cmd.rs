//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
use xrl::ViewId;

use std::str::FromStr;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug)]
pub enum ParseCommandError {
    /// Didnt expect a command to take an argument.
    UnexpectedArgument,
    /// The given command expected an argument.
    ExpectedArgument {
        cmd: String,
        expected: usize,
        found: usize,
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

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, Self::Err> {
        match &s[..] {
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
            "md" | "move-down" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::lines, 
                                                                forward: true, 
                                                                extend: false
                                                                }
                                                   )),
            "mu" | "move-up" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::lines, 
                                                                forward: false, 
                                                                extend: false
                                                                }
                                                   )),
            "mr" | "move-right" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::characters, 
                                                                forward: true, 
                                                                extend: false
                                                                }
                                                   )),
            "ml" | "move-left" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::characters, 
                                                                forward: false, 
                                                                extend: false
                                                                }
                                                   )),
            "pd" | "page-down" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::pages, 
                                                                forward: true, 
                                                                extend: false
                                                                }
                                                   )),
            "pu" | "page-up" => Ok(Command::RelativeMove(
                                                    RelativeMove{
                                                                by: RelativeMoveDistance::pages, 
                                                                forward: false, 
                                                                extend: false
                                                                }
                                                   )),
            "bof" | "beginning-of-file" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::bof,
                                                                extend: false
                                                                }
                                                   )),
            "eof" | "end-of-file" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::eof,
                                                                extend: false
                                                                }
                                                   )),
            "bol" | "beginning-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::bol,
                                                                extend: false
                                                                }
                                                   )),
            "eol" | "end-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::eol,
                                                                extend: false
                                                                }
                                                   )),
            "ln" | "line-numbers" => Ok(Command::ToggleLineNumbers),
            "op" | "open-prompt" | "show_overlay" => Ok(Command::OpenPrompt),
            command => {
                let mut parts: Vec<&str> = command.split(' ').collect();

                let cmd = parts.remove(0);
                match cmd {
                    "t" | "theme" => {
                        if parts.is_empty() {
                            Err(ParseCommandError::ExpectedArgument {
                                cmd: "theme".into(),
                                expected: 1,
                                found: 0,
                            })
                        } else if parts.len() > 1 {
                            Err(ParseCommandError::TooManyArguments {
                                cmd: cmd.to_owned(),
                                expected: 1,
                                found: parts.len(),
                            })
                        } else {
                            Ok(Command::SetTheme(parts[0].to_owned()))
                        }
                    }
                    "o" | "open" => {
                        if parts.is_empty() {
                            Ok(Command::Open(None))
                        } else if parts.len() > 1 {
                            Err(ParseCommandError::UnexpectedArgument)
                        } else {
                            Ok(Command::Open(Some(parts[0].to_owned())))
                        }
                    }
                    _ => Err(ParseCommandError::UnknownCommand(command.into())),
                }
            }
        }
    }
}
