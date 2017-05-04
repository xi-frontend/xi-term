#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy))]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate termion;

mod core;
mod cursor;
mod errors;
mod input;
mod line;
mod operation;
mod screen;
mod update;
mod view;

use error_chain::ChainedError;

use core::Core;
use errors::*;
use input::Input;
use screen::Screen;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();

        writeln!(stderr, "error: {}", e).unwrap();

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).unwrap();
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).unwrap();
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let xi = clap_app!(
        xi =>
        (about: "The Xi Editor")
        (@arg core: -c --core +takes_value "Specify binary to use for the backend")
        (@arg file: +required "File to edit"));

    let matches = xi.get_matches();
    let core_exe = matches.value_of("core").unwrap_or("xi-core");
    let file = matches.value_of("file").unwrap();
    let mut core = Core::new(core_exe);
    let mut screen = Screen::new()?;
    let mut input = Input::new();
    input.run();
    screen.init()?;
    core.open(file)?;
    core.scroll(0, screen.size.1 as u64)?;

    loop {
        if let Ok(event) = input.try_recv() {
            if let Err(e) = input::handle(&event, &mut core) {
                log_error(e);
            }
        } else {
            if let Err(e) = screen.update(&mut core) {
                log_error(e);
            }
        }
    }
}

fn log_error<E: ChainedError>(e: E) {
    error!("error: {}", e);
    for e in e.iter().skip(1) {
        error!("caused by: {}", e);
    }
}
