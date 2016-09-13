#![feature(plugin)]
#![plugin(clippy)]
// #![deny(clippy_pedantic)]
extern crate termion;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate log4rs;


mod core;

use termion::{clear, cursor};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use std::io::{stdout, Write, stdin};
use std::sync::mpsc;
use std::{thread, time};
use core::Core;

struct Screen {
    stdout: termion::raw::RawTerminal<std::io::Stdout>,
    size: (u16, u16),
}


impl Screen {

    fn new() -> Screen {
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout, "{}", clear::All).unwrap();
        stdout.flush().unwrap();
        Screen {
            size: termion::terminal_size().unwrap(),
            stdout: stdout,
        }
    }

    fn redraw(&mut self, lines: Vec<String>) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();
        for (_, line) in lines.into_iter().enumerate() {
            self.stdout.write_all(line.as_bytes()).unwrap();
            write!(self.stdout, "{}", cursor::Left(self.size.0)).unwrap();
        }
        self.stdout.flush();
    }

}

fn update_screen(core: &mut Core, screen: &mut Screen) {
    if let Ok(update_msg) = core.update_rx.try_recv() {
        let update = update_msg.as_object().unwrap().get("update").unwrap().as_object().unwrap();
        let lines = update.get("lines").unwrap().as_array().unwrap().into_iter().map(|line| line.as_array().unwrap()[0].as_str().unwrap().to_string()).collect();
        screen.redraw(lines);
    } else {
        thread::sleep_ms(10);
    }
}

pub struct Input {
    tx: mpsc::Sender<termion::event::Event>,
    rx: mpsc::Receiver<termion::event::Event>,
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
            for event_res in stdin().events() {
                match event_res {
                    Ok(event) => {
                        info!("event: {:?}", event);
                        tx.send(event).unwrap();
                    },
                    Err(err) => {
                        error!("{:?}", err);
                    }
                }
            }
        });
    }
}



fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let mut core = Core::new("../xi-editor/rust/target/debug/xi-core");
    let mut screen = Screen::new();
    let mut input = Input::new();
    input.run();
    loop {
        if let Ok(event) = input.rx.try_recv() {
            match event {
                termion::event::Event::Key(key) => {
                    match key {
                        termion::event::Key::Char(c) => {
                            core.char(c);
                        },
                        termion::event::Key::Ctrl(c) => {
                            if c == 'c' {
                                info!("received ^C: exiting");
                                return;
                            }
                        },
                        _ => {
                            error!("unsupported key event");
                        }
                    }
                },
                termion::event::Event::Mouse(_) => {
                    error!("mouse events are not supported yet");
                }
                _ => {
                    error!("unsupported event");
                }
            }
        } else {
            update_screen(&mut core, &mut screen);
        }
    }
}
