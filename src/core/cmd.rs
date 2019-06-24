//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
use xrl::ViewId;

use std::str::FromStr;

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
    /// Move cursor left.
    MoveLeft,
    /// Move cursor right.
    MoveRight,
    /// Move cursor up.
    MoveUp,
    /// Move cursor down.
    MoveDown,
    /// Page down
    PageDown,
    /// Page up
    PageUp,
    /// Change the syntax theme.
    SetTheme(String),
    /// Toggle displaying line numbers.
    ToggleLineNumbers,
    /// Open prompt for user-input
    OpenPrompt,
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
            "s" | "save" => Ok(Command::Save(None)),
            "q" | "quit" | "exit" => Ok(Command::Quit),
            "b" | "back" | "left_delete" => Ok(Command::Back),
            "d" | "delete" | "right_delete" => Ok(Command::Delete),
            "bn" | "next-buffer" | "next_view" => Ok(Command::NextBuffer),
            "bp" | "prev-buffer" | "prev_view" => Ok(Command::PrevBuffer),
            "pd" | "page-down" => Ok(Command::PageDown),
            "pu" | "page-up" => Ok(Command::PageUp),
            "ml" | "move-left" => Ok(Command::MoveLeft),
            "mr" | "move-right" => Ok(Command::MoveRight),
            "mu" | "move-up" => Ok(Command::MoveUp),
            "md" | "move-down" => Ok(Command::MoveDown),
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
