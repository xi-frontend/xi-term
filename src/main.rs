#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
// #![deny(clippy_pedantic)]
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate serde_json;
extern crate termion;

mod core;
mod line;
mod update;
mod screen;

use std::env;
use std::io::stdin;
use std::sync::mpsc;
use std::thread;

use termion::input::TermRead;

use core::Core;
use screen::Screen;

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

impl Default for Input {
	fn default() -> Self {
		Self::new()
	}
}

fn main() {
    log4rs::init_file("log_config.yaml", Default::default()).unwrap();
    let mut core = Core::new("xi-core");
    let mut screen = Screen::new();
    let mut input = Input::new();
    input.run();
    screen.init();
    core.scroll(0, screen.size.1 as u64 - 2);

    let mut current_file: Option<String> = None;
    if let Some(filename) = env::args().nth(1) {
        core.open(filename.as_str());
        current_file = Some(filename);
    }

    loop {
        if let Ok(event) = input.rx.try_recv() {
            match event {
                termion::event::Event::Key(key) => {
                    match key {
                        termion::event::Key::Char(c) => {
                            core.char(c);
                        },
                        termion::event::Key::Ctrl(c) => {
                            match c {
                                'c' => {
                                    info!("received ^C: exiting");
                                    return;
                                },
                                'w' => {
                                    info!("received ^W: writing current file");
                                    if let Some(ref filename) = current_file {
                                        core.save(filename.as_str());
                                    } else {
                                        error!("no file to save");
                                    }
                                },
                                _ => {}
                            }
                        },
                        termion::event::Key::Backspace => {
                            core.del();
                        },
                        termion::event::Key::Left => {
                            core.left();
                        },
                        termion::event::Key::Right => {
                            core.right();
                        },
                        termion::event::Key::Up => {
                            core.up();
                        },
                        termion::event::Key::Down => {
                            core.down();
                        },
                        termion::event::Key::PageUp => {
                            core.page_up();
                        },
                        termion::event::Key::PageDown => {
                            core.page_down();
                        },
                        _ => {
                            error!("unsupported key event");
                        }
                    }
                },
                termion::event::Event::Mouse(e) => {
                    match e {
                        termion::event::MouseEvent::Press(_, y, x) => {
                            core.click(x as u64 - 1, y as u64 - 1);
                        },
                        termion::event::MouseEvent::Release(_, _) => {},
                        termion::event::MouseEvent::Hold(y, x) => {
                            core.drag(x as u64 - 1, y as u64 - 1);
                        },
                    }
                },
                _ => {
                    error!("unsupported event");
                }
            }
        } else {
            screen.update(&mut core);
        }
    }
}
