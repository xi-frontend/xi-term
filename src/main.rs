#![feature(rustc_private)]
extern crate termion;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate log4rs;


mod core;

use termion::{color, style, clear, cursor};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use std::io::{stdin, stdout, Write, Read};
use std::io;
use core::Core;
use std::{thread, time};
use std::sync::mpsc;

struct Screen<W: Write> {
    stdout: W,
    x: u16,
    y: u16,
    exit: bool,
}

fn init<W: Write>(mut stdout: W, x: u16, y: u16) -> Screen<W> {
    write!(stdout, "{}", clear::All).unwrap();
    stdout.flush();
    Screen {
        x: x,
        y: y,
        stdout: stdout,
        exit: false,
    }
}

impl<W: Write> Screen<W> {
    fn handle_update(&mut self, lines: Vec<String>, x: u16, y: u16) {
        write!(self.stdout, "{}", termion::clear::All);
        write!(self.stdout, "{}", cursor::Up(self.y));
        for (i, line) in lines.into_iter().enumerate() {
            self.stdout.write_all(line.as_bytes());
            write!(self.stdout, "{}", cursor::Left(self.x));
        }
        self.stdout.flush();
    }
}

pub struct Input {
    tx: mpsc::Sender<Option<termion::event::Key>>,
    rx: mpsc::Receiver<Option<termion::event::Key>>
}

impl Input {
    pub fn new() -> Input {
        let (tx, rx) = mpsc::channel();
        Input {
            tx: tx,
            rx: rx,
        }
    }

    pub fn run(&mut self) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            for k in stdin().keys() {
                tx.send(Some(k.unwrap()));
            }
        });
    }

    pub fn into_iter(&mut self) -> InputIntoIterator {
        InputIntoIterator {
            input: self
        }
    }
}

pub struct InputIntoIterator<'a> {
    input: &'a mut Input,
}

impl <'a>Iterator for InputIntoIterator<'a> {

    type Item = Option<termion::event::Key>;

    fn next(&mut self) -> Option<Option<termion::event::Key>> {
        let data = self.input.rx.try_recv();
        if data.is_ok() {
            return Some(data.unwrap());
        }
        else {
            return Some(None);
        }
    }
}

fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let mut core = Core::new("../xi-editor/rust/target/debug/xi-core");
    let mut stdout = stdout().into_raw_mode().unwrap();
    let size = termion::terminal_size().unwrap();
    let mut screen = init(stdout, size.0, size.1);

    let mut input = Input::new();
    input.run();
    let mut keys = input.into_iter();
    loop {
        if let Some(key) = keys.next().unwrap() {
            match key {
                termion::event::Key::Char(c) => {
                    &mut core.char(c);
                }
                _ => {}
            }
        }
        if let Ok(update_msg) = core.update_rx.try_recv() {
            let update = update_msg.as_object().unwrap().get("update").unwrap().as_object().unwrap();
            // first_line = dict.get("first_line").unwrap().as_u64().unwrap();
            // height = dict.get("height").unwrap().as_u64().unwrap();
            let lines = update.get("lines").unwrap().as_array().unwrap().into_iter().map(
                |line| line.as_array().unwrap()[0].as_str().unwrap().to_string()).collect();
            let scrollto = update.get("scrollto").unwrap().as_array().unwrap();
            screen.handle_update(lines, scrollto[0].as_u64().unwrap() as u16, scrollto[1].as_u64().unwrap() as u16);
        }
        thread::sleep_ms(10);
    }
}
