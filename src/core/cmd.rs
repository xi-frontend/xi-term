use xrl::ViewId;

use std::str::FromStr;

/// Command system for xi-term.
/// A command represents a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
#[derive(Debug)]
pub enum Command {
    /// Close the CommandPrompt
    Cancel,
    /// Quit editor.
    Quit,
    /// Save the current file buffer
    Save(Option<ViewId>),
    /// Open A new file.
    Open(Option<String>),
    /// Change the syntax theme.
    SetTheme(String),
    Invalid(String),
}

pub enum ParseCommandError {
    NoTheme,
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, Self::Err> {
        // TODO: Clean this up.
        match &s[..] {
            "s" | "save" => Ok(Command::Save(None)),
            "q" | "quit" => Ok(Command::Quit),
            command => {
                let mut parts: Vec<&str> = command.split(" ").collect();
                let cmd = parts.remove(0);
                match cmd {
                    "t" | "theme" => {
                        if parts.len() == 0 {
                            Err(ParseCommandError::NoTheme)
                        } else {
                            Ok(Command::SetTheme(parts[0].to_owned()))
                        }
                    },
                    "o" | "open" => {
                        if parts.len() == 0 {
                            Ok(Command::Open(None))
                        } else {
                            Ok(Command::Open(Some(parts[0].to_owned())))
                        }
                    }
                    _ => Ok(Command::Invalid(command.into()))
                }
            }
        }
    }
}
