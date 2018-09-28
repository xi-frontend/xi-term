//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform, currently
//! commands can only be input through the CommandPrompt. Vim style.

use xrl::ViewId;

use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    /// Close the CommandPrompt.
    Cancel,
    /// Quit editor.
    Quit,
    /// Save the current file buffer.
    Save(Option<ViewId>),
    /// Open A new file.
    Open(Option<String>),
    /// Change the syntax theme.
    SetTheme(String),
}

#[derive(Debug)]
pub enum ParseCommandError {
    /// Didnt expect a command to take an argument.
    UnexpectedArgument,
    /// The given command expected an argument.
    ExpectedArgument { cmd: String, expected: usize , found: usize},
    /// The given command was given to many arguments.
    ToManyArguments{ cmd: String, expected: usize, found: usize},
    /// Invalid input was received.
    UnknownCommand(String),
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, Self::Err> {
        match &s[..] {
            "s" | "save" => Ok(Command::Save(None)),
            "q" | "quit" => Ok(Command::Quit),
            command => {
                let mut parts: Vec<&str> = command.split(" ").collect();

                let cmd = parts.remove(0);
                match cmd {
                    "t" | "theme" => {
                        if parts.len() == 0 {
                            Err(ParseCommandError::ExpectedArgument {
                                cmd: "theme".into(),
                                expected: 1,
                                found: 0
                            })
                        } else if parts.len() > 1 {
                            Err(ParseCommandError::ToManyArguments {
                                cmd: cmd.to_owned(),
                                expected: 1,
                                found: parts.len()
                            })
                        } else {
                            Ok(Command::SetTheme(parts[0].to_owned()))
                        }
                    },
                    "o" | "open" => {
                        if parts.len() == 0 {
                            Ok(Command::Open(None))
                        } else {
                            if parts.len() > 1 {
                                Err(ParseCommandError::UnexpectedArgument)
                            } else {
                                Ok(Command::Open(Some(parts[0].to_owned())))
                            }
                        }
                    }
                    _ => Err(ParseCommandError::UnknownCommand(command.into()))
                }
            }
        }
    }
}
