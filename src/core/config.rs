use crate::core::{Command, RelativeMove, AbsoluteMove, ExpandLinesDirection};
use termion::event::{Event, Key};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use std::str::FromStr;

pub type Keymap = HashMap<Event, Command>;

#[derive(Debug, Serialize, Deserialize)]
struct Keybinding {
    keys: Vec<String>,
    command: String,
    // For now, unstructured value
    args: Option<Value>,
    context: Option<Value>,
}

#[derive(Clone)]
pub struct KeybindingConfig {
    pub keymap: Keymap,
    pub config_path: PathBuf
}

impl KeybindingConfig {
    pub fn parse(config_path: &Path) -> Result<KeybindingConfig, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let entries = fs::read_to_string(config_path)?;
        // Read the JSON contents of the file as an instance of `User`.
        let bindings: Vec<Keybinding> = json5::from_str(&entries)?;
        error!("Bindings parsed!");

        let mut keymap = Keymap::new();
        let mut found_cmds = Vec::new();
        for binding in bindings {
            let cmd = match binding.command.as_ref() {
                "move"    => {
                    let args = binding.args.ok_or("move binding incomplete! Missing \"args\"")?;
                    let cmd : RelativeMove = serde_json::from_value(args)?;
                    Command::RelativeMove(cmd)
                },
                "move_to" => {
                    let args = binding.args.ok_or("move_to binding incomplete! Missing \"args\"")?;
                    let cmd : AbsoluteMove = serde_json::from_value(args)?;
                    Command::AbsoluteMove(cmd)
                },
                "select_lines" => {
                    let args = binding.args.ok_or("select_lines binding incomplete! Missing \"args\"")?;
                    let cmd : ExpandLinesDirection = serde_json::from_value(args)?;
                    Command::CursorExpandLines(cmd)
                }
                x =>  match Command::from_str(x) {
                          Ok(cmd) => cmd,
                          // unimplemented command for now
                          Err(_) => continue,
                }
            };
            if found_cmds.contains(&cmd) {
                continue;
            }
            if let Some(binding) = KeybindingConfig::parse_keys(&binding.keys) {
                error!("{:?} = {:?}", cmd, binding);
                keymap.insert(binding, cmd.clone());
                found_cmds.push(cmd);
            } else {
                // warn!("Skipping failed binding");
                continue;
            }
        }

        Ok(KeybindingConfig{keymap: keymap, config_path: config_path.to_owned()})
    }

    fn parse_keys(keys: &Vec<String>) -> Option<Event> {
        if keys.len() != 1 {
            return None;
        }

        let key = &keys[0];
        match key.as_ref() {
            "enter" => Some(Event::Key(Key::Char('\n'))),
            "tab" => Some(Event::Key(Key::Char('\t'))),
            "backspace" => Some(Event::Key(Key::Backspace)),
            "left" => Some(Event::Key(Key::Left)),
            "right" => Some(Event::Key(Key::Right)),
            "up" => Some(Event::Key(Key::Up)),
            "down" => Some(Event::Key(Key::Down)),
            "home" => Some(Event::Key(Key::Home)),
            "end" => Some(Event::Key(Key::End)),
            "pageup" => Some(Event::Key(Key::PageUp)),
            "pagedown" => Some(Event::Key(Key::PageDown)),
            "delete" => Some(Event::Key(Key::Delete)),
            "insert" => Some(Event::Key(Key::Insert)),
            "escape" => Some(Event::Key(Key::Esc)),
            "ctrl+right" => Some(Event::Unsupported(vec![27, 91, 49, 59, 53, 67])),
            "ctrl+left"  => Some(Event::Unsupported(vec![27, 91, 49, 59, 53, 68])),
            "ctrl+home"  => Some(Event::Unsupported(vec![27, 91, 49, 59, 53, 72])),
            "ctrl+end"   => Some(Event::Unsupported(vec![27, 91, 49, 59, 53, 70])),
            "ctrl+shift+right" => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 67])),
            "ctrl+shift+left"  => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 68])),
            "ctrl+shift+up"    => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 65])),
            "ctrl+shift+down"  => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 66])),
            "ctrl+shift+home"  => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 72])),
            "ctrl+shift+end"   => Some(Event::Unsupported(vec![27, 91, 49, 59, 54, 70])),
            "alt+shift+up"     => Some(Event::Unsupported(vec![27, 91, 49, 59, 52, 65])),
            "alt+shift+down"   => Some(Event::Unsupported(vec![27, 91, 49, 59, 52, 66])),
            "shift+up"         => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 65])),
            "shift+down"       => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 66])),
            "shift+right"      => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 67])),
            "shift+left"       => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 68])),
            "shift+pageup"     => Some(Event::Unsupported(vec![27, 91, 53, 59, 50, 126])),
            "shift+pagedown"   => Some(Event::Unsupported(vec![27, 91, 54, 59, 50, 126])),
            "shift+end"        => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 70])),
            "shift+home"       => Some(Event::Unsupported(vec![27, 91, 49, 59, 50, 72])),
            
            // Not yet released
            // "shift+tab" => Some(Event::Key(Key::Backtab)),

            x if x.starts_with("f") => {
                match x[1..].parse::<u8>() {
                    Ok(val) => Some(Event::Key(Key::F(val))),
                    Err(_) => {
                        // warn!("Cannot parse {}", x);
                        None
                    }
                }
            }

            x if x.starts_with("ctrl+") || x.starts_with("alt+") => {
                let is_alt = x.starts_with("alt+");
                let start_length = if is_alt {4} else {5};

                let character;
                // start_length + "shift+x".len() || start_length + "x".len()
                if x.len() != start_length + 7 && x.len() != start_length + 1 {
                    // warn!("Cannot parse {}. Length is = {}, which is neither {} nor {} ", x, x.len(), start_length + 1, start_length + 7);
                    return None
                } else {
                    if x.len() == start_length + 7 {
                        // With "+shift+", so we use an upper case letter
                        character = x.chars().last().unwrap().to_ascii_uppercase();
                    } else {
                        character = x.chars().last().unwrap().to_ascii_lowercase();
                    }                   
                }

                if is_alt {
                    Some(Event::Key(Key::Alt(character)))
                } else {
                    Some(Event::Key(Key::Ctrl(character)))
                }
            }
            
            _ => {
                // warn!("Completely unknown argument {}", x);
                None
            },
        }
    }
}
