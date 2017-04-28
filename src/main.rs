#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

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
extern crate xi_rpc;
mod core;
mod input;
mod line;
mod update;
mod operation;
mod screen;
mod view;

use std::io::BufReader;
use xi_rpc::RpcLoop;
use std::process::{Stdio, Command};
use core::Core;

fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();

    let xi = clap_app!(
        xi =>
            (about: "The Xi Editor")
            (@arg core: -c --core +takes_value "Specify binary to use for the backend")
            (@arg file: +required "File to edit")
    );
    let matches = xi.get_matches();
    let core_exe = matches.value_of("core").unwrap_or("xi-core");
    let file = matches.value_of("file").unwrap();

    let process = Command::new(core_exe)
        .arg("test-file")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .env("RUST_BACKTRACE", "1")
        .spawn()
        .unwrap_or_else(|e| panic!("failed to execute core: {}", e));

    let stdin = process.stdin.unwrap();
    let mut rpc_loop = RpcLoop::new(stdin);
    let mut core = Core::new(rpc_loop.get_peer());

    let stdout = process.stdout.unwrap();
    rpc_loop.mainloop(move || BufReader::new(stdout), &mut core);
}
