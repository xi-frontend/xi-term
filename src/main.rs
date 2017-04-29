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
mod client;
mod view;

use std::io::BufReader;
use xi_rpc::RpcLoop;
use std::process::{Stdio, Command};
use client::Client;
use core::Core;

use std::thread;
use std::time::Duration;

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
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .env("RUST_BACKTRACE", "1")
        .spawn()
        .unwrap_or_else(|e| panic!("failed to execute core: {}", e));


    let mut rpc_loop = RpcLoop::new(process.stdin.unwrap());
    let client = Client::new(rpc_loop.get_peer());

    let mut core = Core::new(client.clone());

    let xi_stdout = process.stdout.unwrap();
    let t = thread::spawn(move || {
            rpc_loop.mainloop(move || BufReader::new(xi_stdout), &mut core)
    });

    info!("{:?}", client.new_view(Some(file)));

    t.join();
}
