#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy))]

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate termion;

mod core;
mod input;
mod line;
mod update;
mod operation;
mod screen;
mod view;

use core::Core;
use input::Input;
use screen::Screen;

fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let xi = clap_app!(xi =>
                       (about: "The Xi Editor")
                       (@arg core: -c --core +takes_value "Specify binary to use for the backend")
                       (@arg file: +required "File to edit")
                      );
    let matches = xi.get_matches();
    let core_exe = matches.value_of("core").unwrap_or("xi-core");
    let file = matches.value_of("file").unwrap();
    let mut core = Core::new(core_exe, file);
    let mut screen = Screen::new();
    let mut input = Input::new();
    input.run();
    screen.init();
    core.scroll(0, screen.size.1 as u64 - 2);

    loop {
        if let Ok(event) = input.try_recv() {
            input::handle(&event, &mut core);
        } else {
            screen.update(&mut core);
        }
    }
}
