use std;
use std::io::stdin;
use std::sync::mpsc;
use std::thread;

use termion;
use termion::input::TermRead;

use core::Core;

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

    pub fn try_recv(&mut self) -> Result<termion::event::Event, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

pub fn handle(event: &termion::event::Event, core: &mut Core) {
    match *event {
        termion::event::Event::Key(key) => {
            match key {
                termion::event::Key::Char(c) => {
                    core.char(c);
                },
                termion::event::Key::Ctrl(c) => {
                    match c {
                        'c' => {
                            info!("received ^C: exiting");
                            std::process::exit(0);
                        },
                        'w' => {
                            info!("received ^W: writing current file");
                            core.save();
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
}
