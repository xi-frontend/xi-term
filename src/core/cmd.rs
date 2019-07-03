//! Command system for xi-term. A command represents
//! a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
use xrl::ViewId;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::KeymapEntry;
use crate::widgets::CommandPromptMode;
use std::collections::HashMap;

pub type ParserMap = HashMap<&'static str, CommandParser>;

#[derive(Clone)]
pub struct CommandParser {
    pub keybinding: Option<String>,
    pub from_prompt: fn(add_args: Option<&str>) -> Result<Command, ParseCommandError>,
    // pub to_prompt: fn() -> String,
    pub subcommands: Vec<&'static str>,
    pub from_keymap_entry: Option<fn (val: KeymapEntry) -> Result<Command, ParseCommandError>>,
}

pub fn get_parser_map() -> ParserMap {
    let mut map = HashMap::new();

    map.insert("select_all", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::SelectAll),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("close", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::CloseCurrentView),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("copy", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::CopySelection),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("cut", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::CutSelection),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("paste", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Paste),
        subcommands: vec![],
        from_keymap_entry: None});
    // "fue" | 
    map.insert("find_under_expand", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::FindUnderExpand),
        subcommands: vec![],
        from_keymap_entry: None});
    // "fn" | 
    map.insert("find_next", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::FindNext),
        subcommands: vec![],
        from_keymap_entry: None});
    // "fp" | 
    map.insert("find_prev", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::FindPrev),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("hide_overlay", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Cancel),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("save", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Save(None)),
        subcommands: vec![],
        from_keymap_entry: None});
    // "q" | "quit"
    map.insert("exit", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Quit),
        subcommands: vec![],
        from_keymap_entry: None});
    // "b" | "back" | 
    map.insert("left_delete", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Back),
        subcommands: vec![],
        from_keymap_entry: None});
    // "d" | "delete" | 
    map.insert("right_delete", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Delete),
        subcommands: vec![],
        from_keymap_entry: None});
    // "bn" | "next-buffer" | 
    map.insert("next_view", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::NextBuffer),
        subcommands: vec![],
        from_keymap_entry: None});
    // "bp" | "prev-buffer" | 
    map.insert("prev_view", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::PrevBuffer),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("undo", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Undo),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("redo", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::Redo),
        subcommands: vec![],
        from_keymap_entry: None});
    // "ln" | 
    map.insert("line-numbers", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::ToggleLineNumbers),
        subcommands: vec![],
        from_keymap_entry: None});
    // "op" | 
    map.insert("open-prompt", CommandParser{ keybinding: None,
        from_prompt: |_| Ok(Command::OpenPrompt(CommandPromptMode::Command)),
        subcommands: vec![],
        from_keymap_entry: None});
    map.insert("select_all", CommandParser{ keybinding: None,
        from_prompt:       |_| Ok(Command::SelectAll), 
        subcommands:      vec![], 
        from_keymap_entry: None});
    map.insert("move", CommandParser{ keybinding: None,
        from_prompt: RelativeMove::from_prompt, 
        subcommands: vec!["left", "right", "down", "up", "wordleft", "wordright", 
                          "wendleft", "wendright", "subwordleft", "subwordright", 
                          "subwendleft", "subwendright", "page-down", "page-up"], 
        from_keymap_entry: Some(|val| {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move".to_string()})?;
                let cmd : RelativeMove = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::RelativeMove(cmd))})});
    map.insert("move_to", CommandParser{ keybinding: None,
        from_prompt: AbsoluteMove::from_prompt, 
        subcommands: vec!["bof", "eof", "bol", "eol", "brackets", "<linenumber>"],
        from_keymap_entry: Some(|val| {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move_to".to_string()})?;
                let cmd : AbsoluteMove = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::AbsoluteMove(cmd))})});
    map.insert("select_lines", CommandParser{ keybinding: None,
        from_prompt: ExpandLinesDirection::from_prompt,
        subcommands: vec!["above", "below"],
        from_keymap_entry: Some(|val| {
                let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "select_lines".to_string()})?;
                let cmd : ExpandLinesDirection = serde_json::from_value(args).map_err(|_| ParseCommandError::UnexpectedArgument)?;
                Ok(Command::CursorExpandLines(cmd))})});
    // "t"
    map.insert("theme", CommandParser{ keybinding: None,
        from_prompt: |args| {
                         let theme = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "theme".to_string()})?;
                         Ok(Command::SetTheme(theme.to_string()))
                     }, 
        subcommands:      vec!["<themename>"],  // TODO: Get in here the available themes
        from_keymap_entry: None});
    // "f"
    map.insert("find", CommandParser{ keybinding: None,
        from_prompt: |args| {
                        let needle = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "find".to_string()})?;
                        FindConfig::from_prompt(Some(needle))
                     },
        subcommands:      vec!["<needle>"], 
        from_keymap_entry: None});
    // "o"
    map.insert("open", CommandParser{ keybinding: None,
        from_prompt: |args| {
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
                     },
        subcommands:      vec!["<needle>"], 
        from_keymap_entry: None});
    map.insert("show_overlay", CommandParser{ keybinding: None,
        from_prompt: |_| { Err(ParseCommandError::UnexpectedArgument) },
        subcommands:      vec![], 
        from_keymap_entry: Some(|val| {
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
        })
    });
    map.insert("show_panel", CommandParser{ keybinding: None,
        from_prompt: |_| { Err(ParseCommandError::UnexpectedArgument) },
        subcommands:      vec![], 
        from_keymap_entry: Some(|val| {
                   let args = val.args.ok_or(ParseCommandError::ExpectedArgument{cmd: "show_panel".to_string()})?;
                    match args.get("panel") {
                        None => Err(ParseCommandError::UnexpectedArgument),
                        Some(value) => match value {
                                            Value::String(x) if x == "find" => Ok(Command::OpenPrompt(CommandPromptMode::Find)),
                                            _ => Err(ParseCommandError::UnexpectedArgument),
                                       }
                    }
        })
    });

    map
}

pub trait FromPrompt {
    fn from_prompt(vals: Option<&str>) -> Result<Command, ParseCommandError>;
}

#[serde(rename_all = "snake_case")] 
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum RelativeMoveDistance {
    /// Move only one character
    Characters,
    /// Move a line
    Lines,
    /// Move to new word
    Words,
    /// Move to end of word
    WordEnds,
    /// Move to new subword
    Subwords,
    /// Move to end of subword
    SubwordEnds,
    /// Move a page
    Pages,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RelativeMove {
    pub by: RelativeMoveDistance,
    pub forward: bool,
    #[serde(default)]
    pub extend: bool
}

impl FromPrompt for RelativeMove {
    fn from_prompt(args: Option<&str>) -> Result<Command, ParseCommandError> {
        let args = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move".to_string()})?;
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
                                            by: RelativeMoveDistance::Lines, 
                                            forward: true, 
                                            extend
                                            }
                               )),
            "u" | "up" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::Lines, 
                                            forward: false, 
                                            extend
                                            }
                               )),
            "r" | "right" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::Characters, 
                                            forward: true, 
                                            extend
                                            }
                               )),
            "l" | "left" => Ok(Command::RelativeMove(
                                RelativeMove{
                                            by: RelativeMoveDistance::Characters, 
                                            forward: false, 
                                            extend
                                            }
                               )),
            "pd" | "page-down" => Ok(Command::RelativeMove(
                                        RelativeMove{
                                                    by: RelativeMoveDistance::Pages, 
                                                    forward: true, 
                                                    extend
                                                    }
                                       )),
            "pu" | "page-up" => Ok(Command::RelativeMove(
                                        RelativeMove{
                                                    by: RelativeMoveDistance::Pages, 
                                                    forward: false, 
                                                    extend
                                                    }
                                       )),
            command => Err(ParseCommandError::UnknownCommand(command.into()))
        }
    }
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum AbsoluteMovePoint {
    /// Beginning of file
    BOF,
    /// End of file
    EOF,
    /// Beginning of line
    BOL,
    /// End of line
    EOL,
    /// Enclosing brackets
    Brackets,
    /// Line number
    Line(u64)
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AbsoluteMove {
    pub to: AbsoluteMovePoint,
    #[serde(default)]
    pub extend: bool
}

impl FromPrompt for AbsoluteMove {
    fn from_prompt(args: Option<&str>) -> Result<Command, ParseCommandError> {
        let args = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "move_to".to_string()})?;
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
                                                                to: AbsoluteMovePoint::BOF,
                                                                extend
                                                                }
                                                   )),
            "eof" | "end-of-file" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::EOF,
                                                                extend
                                                                }
                                                   )),
            "bol" | "beginning-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::BOL,
                                                                extend
                                                                }
                                                   )),
            "eol" | "end-of-line" => Ok(Command::AbsoluteMove(
                                                    AbsoluteMove{
                                                                to: AbsoluteMovePoint::EOL,
                                                                extend
                                                                }
                                                   )),

            command => {
                let number = command.parse::<u64>().map_err(|_| ParseCommandError::UnknownCommand(command.into()))?;
                Ok(Command::AbsoluteMove(
                                AbsoluteMove{
                                            to: AbsoluteMovePoint::Line(number),
                                            extend: false
                                            }
                                )
                )
            }
        }

    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ExpandLinesDirection {
    pub forward: bool
}

impl FromPrompt for ExpandLinesDirection {
    fn from_prompt(args: Option<&str>) -> Result<Command, ParseCommandError> {
        let arg = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "select_lines".to_string()})?;
        match arg {
            "a" | "above" => Ok(Command::CursorExpandLines(
                                                    ExpandLinesDirection{forward: false}
                                                   )),
            "b" | "below" => Ok(Command::CursorExpandLines(
                                                    ExpandLinesDirection{forward: true}
                                                   )),
            command => Err(ParseCommandError::UnknownCommand(command.into()))
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct FindConfig {
    pub search_term: String,
    pub case_sensitive: bool,
    pub regex: bool,
    pub whole_words: bool,
}

impl FromPrompt for FindConfig {
    fn from_prompt(args: Option<&str>) -> Result<Command, ParseCommandError> {
        let args = args.ok_or(ParseCommandError::ExpectedArgument{cmd: "find".to_string()})?;
        if args.is_empty() {
            return Err(ParseCommandError::ExpectedArgument{cmd: "find".to_string()})
        }

        let mut search_term = args;
        let mut case_sensitive = false;
        let mut regex = false;
        let mut whole_words = false;

        let argsvec : Vec<&str> = args.splitn(2, ' ').collect();

        if argsvec.len() == 2 && argsvec[0].len() <= 3 {
            // We might have search control characters here
            let control_chars = argsvec[0];

            let mut failed = false;
            let mut shadows = [false, false, false];
            for cc in control_chars.chars() {
                match cc {
                    'c' => shadows[0] = true,
                    'r' => shadows[1] = true,
                    'w' => shadows[2] = true,
                    _ => {
                        // Ooops! This first part is NOT a control-sequence after all. Treat it as normal text
                        failed = true;
                        break;
                    }
                }
            }

            if !failed {
                // Strip away control characters of search_term
                search_term = argsvec[1];
                case_sensitive = shadows[0];
                regex          = shadows[1];
                whole_words    = shadows[2];
            }
        }

        let config = FindConfig{
            search_term: search_term.to_string(),
            case_sensitive,
            regex,
            whole_words,
        };
        Ok(Command::Find(config))
    }
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
    Find(FindConfig),
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
