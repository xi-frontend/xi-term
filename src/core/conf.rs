use std::collections::HashMap;
use core::Command;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use std::str::FromStr;

// The structure is intended to allow any structure of modal or
// sequential keybindings by having flags for wether or not the layer
// is kept after entering a command or not and commands for switching
// to any named layer. The logic for this layer switching needs to be
// implemented in cmd.rs.

// Define a struct to hold the config values
#[derive(Debug, Deserialize)]
pub struct PreSettings {
    success: Option<bool>,
    this: Option<String>,
    keymaps: Option<HashMap<String, HashMap<String, String>>>,
}
#[derive(Debug, Deserialize)]
pub struct Settings {
    layer: String,
    keymaps: HashMap<String, HashMap<String, String>>,
}
// Add methods to that struct
impl Settings {

    // Define a constructor that reads from the main config
    pub fn new() -> Self {
        // Read config file
        let mut configurator = config::Config::default();
        configurator
            .merge(config::File::with_name("test")) // TODO use reasonable file
            .unwrap(); // TODO handle no custom config

        // Write the settings into a struct
        let settings = configurator.try_into::<PreSettings>().unwrap(); // TODO handle syntax error

        // Extend the user settings with defaults (doesn't overwrite)
        //apply_defaults(settings);


        //println!("{:?}", settings);

        // Return the resulting struct
        Settings{
            layer: "base".to_string(),
            keymaps: settings.keymaps.unwrap(),
        }
    }

    fn get_command_string(&self, key: &str) -> String {
        // Check if the string is overridden
        match self.keymaps.get("override").unwrap().get(key) {
            // If override command exists, run that
            Some(command) => return command.clone(),
            // Else, check the layer
            None => {
                match self.keymaps.get(&self.layer).unwrap().get(key) {
                    // If layer command exists, run that
                    Some(command) => return command.clone(),
                    // Else, check if we should use base layer's command
                    None => {
                        match self.keymaps.get(&self.layer).unwrap().get("blank") {
                            // If blank is true fallback to base is off
                            Some(blank) => {
                                // As you can see we are going with a very generous string to bool conversion
                                if blank.contains("t") {
                                    return "".to_string();
                                }
                                else {
                                    match self.keymaps.get("base").unwrap().get(key) {
                                        Some(command) => return command.clone(),
                                        None => return "".to_string(),
                                    }
                                }
                            }
                            // Else fetch command from base layer
                            _ => {
                                match self.keymaps.get("base").unwrap().get(key) {
                                    Some(command) => return command.clone(),
                                    None => return "".to_string(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_command(&self, event: Event) -> Command {
        //Separate by mouse or key event
        match event {
            // Separate by what key event
            // Termion doesn't support modifier combinations,
            // so that potential case is ignored
            Event::Key(key) => match key {
                // Match all ctrl modified
                Key::Ctrl(c) => {
                    return Command::from_str(&self.get_command_string(&("ctrl-".to_string() + &c.to_string()))).unwrap();
                },
                // Match all alt modified
                Key::Alt(c) => {
                    return Command::from_str(&self.get_command_string(&("alt-".to_string() + &c.to_string()))).unwrap();
                },
                // Match all unmodified
                Key::Char(c) => {
                    return Command::from_str(&self.get_command_string(&c.to_string())).unwrap();
                },
                // Handle special keys
                Key::Backspace => {
                    return Command::from_str(&self.get_command_string("backspace")).unwrap();
                },
                Key::Delete => {
                    return Command::from_str(&self.get_command_string("delete")).unwrap();
                },
                Key::Left => {
                    return Command::from_str(&self.get_command_string("left")).unwrap();
                },
                Key::Right => {
                    return Command::from_str(&self.get_command_string("right")).unwrap();
                },
                Key::Up => {
                    return Command::from_str(&self.get_command_string("up")).unwrap();
                },
                Key::Down => {
                    return Command::from_str(&self.get_command_string("down")).unwrap();
                },
                Key::Home => {
                    return Command::from_str(&self.get_command_string("home")).unwrap();
                },
                Key::End => {
                    return Command::from_str(&self.get_command_string("end")).unwrap();
                },
                Key::PageUp => {
                    return Command::from_str(&self.get_command_string("pageup")).unwrap();
                },
                Key::PageDown => {
                    return Command::from_str(&self.get_command_string("pagedown")).unwrap();
                },
                k => {
                    error!("un-handled key {:?}", k);
                    return Command::from_str(&self.get_command_string("")).unwrap();
                }
            }
            k => {
                error!("un-handled event {:?}", k);
                return Command::from_str(&self.get_command_string("")).unwrap();
            }
        }
    }

    // Saved for reference
    fn default_settings() -> PreSettings {
        // The default bindings are fallbacks for all layers
        // (Unless the 'blank' flag is set)
        // therefore only layer flags need to be added here

        // Declare all the base keymap layers
        let mut ctrl = HashMap::new();
        let mut alt = HashMap::new();

        // Set the expected settings to the layers
        ctrl.insert("hold".to_string(), "true".to_string());
        alt.insert("hold".to_string(), "true".to_string());

        // Add the layers to a joining map
        let mut maps = HashMap::new();
        maps.insert("ctrl".to_string(), ctrl);
        maps.insert("alt".to_string(), alt);

        // Return the resulting map
        PreSettings {
            success: None,
            this: None,
            keymaps: Some(maps),
        }
    }

    // Define a "Get map" function that returns a mutable reference to the map

    // Define a "Get value(key)" function to minimise handing the full reference around

    // Define a "Set value(key, value)" as above
}
