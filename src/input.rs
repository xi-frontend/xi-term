use std::io::stdin;
use std::sync::mpsc;
use std::thread;

use termion;
use termion::input::TermRead;

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
