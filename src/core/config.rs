use crate::core::Command;
use termion::event::{Event, Key};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct Keybinding {
    keys: Vec<String>,
    command: String,
    // For now, unstructured value
    args: Option<Value>,
    context: Option<Value>,
}

pub struct KeybindingConfig {
	pub keymap: HashMap<Event, Command>,
	pub config_path: PathBuf
}

impl KeybindingConfig {
	pub fn parse(config_path: &Path) -> Result<KeybindingConfig, Box<std::error::Error + Sync + Send + 'static>> {
        let entries = fs::read_to_string(config_path)?;
	    // Read the JSON contents of the file as an instance of `User`.
	    let bindings: Vec<Keybinding> = json5::from_str(&entries)?;
    	error!("Bindings parsed!");

    	let mut keymap = HashMap::new();
    	let mut found_cmds = Vec::new();
    	for binding in bindings {
    		let cmd = match Command::from_str(&binding.command) {
    			Ok(cmd) => cmd,
    			// unimplemented command for now
    			Err(_) => continue,
    		};
    		error!("{:?} = {:?}", cmd, binding.keys);
    		if found_cmds.contains(&cmd) {
    			continue;
    		}
    		let binding = KeybindingConfig::parse_keys(&binding.keys).ok_or("Could not parse keybindings - config")?;
    		keymap.insert(binding, cmd.clone());
    		found_cmds.push(cmd);
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

			x if x.starts_with("f") => {
				match x[1..].parse::<u8>() {
					Ok(val) => Some(Event::Key(Key::F(val))),
					Err(_) => {
						warn!("Cannot parse {}", x);
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
					warn!("Cannot parse {}. Length is = {}, which is neither {} nor {} ", x, x.len(), start_length + 1, start_length + 7);
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
			
			x => {
				warn!("Completely unknown argument {}", x);
				None
			},
		}
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    
	    

	}
}