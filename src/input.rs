use std;
use std::io::stdin;
use std::sync::mpsc;
use std::thread;

use termion::event::Event;
use termion::event::Key;
use termion::event::MouseButton;
use termion::event::MouseEvent;
use termion::input::TermRead;

use core::Core;
use errors::*;

pub struct Input {
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
}

impl Input {
    pub fn new() -> Input {
        let (tx, rx) = mpsc::channel();
        Input { tx: tx, rx: rx }
    }

    pub fn run(&mut self) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            info!("waiting for input events");
            for event_res in stdin().events() {
                match event_res {
                    Ok(event) => {
                        tx.send(event).unwrap();
                    }
                    Err(err) => {
                        error!("{:?}", err);
                    }
                }
            }
            info!("stop waiting for input events");
        });
    }

    pub fn try_recv(&mut self) -> ::std::result::Result<Event, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

pub fn handle(event: &Event, core: &mut Core) -> Result<()> {
    match *event {
        Event::Key(key) => match key {
            Key::Char(c) => {
                core.char(c)?;
            }
            Key::Ctrl(c) => match c {
                'c' => {
                    info!("received ^C: exiting");
                    std::process::exit(0);
                }
                'w' => {
                    info!("received ^W: writing current file");
                    core.save()?;
                }
                _ => {
                    bail!(ErrorKind::InputError);
                }
            },
            Key::Backspace => {
                core.del()?;
            }
            Key::Left => {
                core.left()?;
            }
            Key::Right => {
                core.right()?;
            }
            Key::Up => {
                core.up()?;
            }
            Key::Down => {
                core.down()?;
            }
            Key::PageUp => {
                core.page_up()?;
            }
            Key::PageDown => {
                core.page_down()?;
            }
            _ => {
                error!("unsupported key event");
                bail!(ErrorKind::InputError);
            }
        },
        Event::Mouse(mouse_event) => match mouse_event {
            MouseEvent::Press(press_event, y, x) => match press_event {
                MouseButton::Left => {
                    core.click(u64::from(x) - 1, u64::from(y) - 1)?;
                }
                MouseButton::WheelUp => {
                    core.up()?;
                }
                MouseButton::WheelDown => {
                    core.down()?;
                }
                _ => {}
            },
            MouseEvent::Release(..) => {}
            MouseEvent::Hold(y, x) => {
                core.drag(u64::from(x) - 1, u64::from(y) - 1)?;
            }
        },
        _ => {
            error!("unsupported event");
            bail!(ErrorKind::InputError);
        }
    }
    Ok(())
}
