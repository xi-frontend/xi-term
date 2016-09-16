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

    fn redraw(&mut self, mut update: &Update) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();
        for (_, line) in update.lines.iter().enumerate() {
            write!(self.stdout, "{}", cursor::Left(self.size.0)).unwrap();
            self.stdout.write_all(line.text.as_bytes()).unwrap();
        }
        // this is pretty weird
        write!(self.stdout, "{}", cursor::Goto(update.scroll_to.1 as u16 + 1, update.scroll_to.0 as u16 + 1)).unwrap();
        self.stdout.flush();
    }

    fn init(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Up(self.size.1)).unwrap();
        self.stdout.flush();
    }

}

struct Update {
    lines: Vec<Line>,
    height: u64,
    scroll_to: (u64, u64),
    first_line: u64
}

impl Update {
    fn from_value(value: &serde_json::Value) -> Update {
        let object = value.as_object().unwrap();
        let scroll_to = object.get("scrollto").unwrap().as_array().unwrap();
        let mut lines: Vec<Line> = vec![];
        for line in object.get("lines").unwrap().as_array().unwrap().iter() {
            lines.push(Line::from_value(line));
        }
        info!("{:?}", object);
        Update {
            height: object.get("height").unwrap().as_u64().unwrap(),
            first_line: object.get("first_line").unwrap().as_u64().unwrap(),
            lines: lines,
            scroll_to: (scroll_to[0].as_u64().unwrap(), scroll_to[1].as_u64().unwrap()),
        }
    }
}

struct Line {
    text: String,
    selection: Option<(u64, u64)>,
    cursor: Option<u64>,
}

impl Line {
    fn from_value(value: &serde_json::Value) -> Line {
        let line_arr = value.as_array().unwrap();
        let mut line = Line {
            text: line_arr[0].as_str().unwrap().to_string(),
            cursor: None,
            selection: None,
        };
        for annotation in line_arr.iter().skip(1).map(|a| a.as_array().unwrap()) {
            match annotation[0].as_str().unwrap() {
                "cursor" => {
                    line.cursor = Some(annotation[1].as_u64().unwrap());
                },
                "sel" => {
                    line.selection = Some((annotation[1].as_u64().unwrap(), annotation[2].as_u64().unwrap()));
                },
                _ => {
                    error!("unknown annotation");
                }
            }
        }
        line
    }
}

fn update_screen(core: &mut Core, screen: &mut Screen) {
    if let Ok(msg) = core.update_rx.try_recv() {
        let update = Update::from_value(msg.as_object().unwrap().get("update").unwrap());
        screen.redraw(&update);
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
    screen.init();
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
