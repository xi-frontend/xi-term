extern crate termion;
extern crate serde_json;

mod core;

use termion::{color, style};
use std::io;
use core::Core;
use std::{thread, time};

fn main() {
    let mut core = Core::new("../xi-editor/rust/target/debug/xi-core");
    core.open("foo.txt");
    core.char('h');
    core.char('e');
    core.char('l');
    core.char('l');
    core.char('o');
    core.char(' ');
    core.char('w');
    core.char('o');
    core.char('r');
    core.char('l');
    core.char('d');
    core.char('!');
    core.insert_newline();
    thread::sleep(time::Duration::from_millis(1000));
}
