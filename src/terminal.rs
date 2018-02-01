use std::io::{self, Stdout};
use std::thread::{sleep, spawn};
use std::time::Duration;

use futures::{Async, Poll, Sink, Stream};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

use termion::terminal_size;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::event::Event;
use termion::input::TermRead;

use errors::*;

pub struct Terminal {
    size: UnboundedReceiver<(u16, u16)>,
    stdin: UnboundedReceiver<Event>,
    stdout: MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let (stdin_tx, stdin_rx) = unbounded();
        let (size_tx, size_rx) = unbounded();
        let stdout = MouseTerminal::from(AlternateScreen::from(io::stdout()
            .into_raw_mode()
            .chain_err(|| "failed to put terminal into raw mode")?));

        let term = Terminal {
            stdin: stdin_rx,
            size: size_rx,
            stdout: stdout,
        };

        Terminal::start_stdin_listening(stdin_tx);
        Terminal::start_size_listening(size_tx);
        Ok(term)
    }

    fn start_stdin_listening(tx: UnboundedSender<Event>) {
        let mut tx = tx;
        spawn(move || {
            info!("waiting for input events");
            for event_res in io::stdin().events() {
                match event_res {
                    // TODO: at least log the errors
                    Ok(event) => {
                        let _ = tx.start_send(event).unwrap();
                        let _ = tx.poll_complete().unwrap();
                    }
                    Err(e) => error!("{}", e),
                }
            }
            info!("stop waiting for input events");
        });
    }

    fn start_size_listening(tx: UnboundedSender<(u16, u16)>) {
        let mut tx = tx;
        spawn(move || {
            let mut current_size = (0, 0);
            info!("waiting for resize events");
            loop {
                match terminal_size() {
                    Ok(new_size) => if new_size != current_size {
                        info!(
                            "terminal resized (from {:?} to {:?})",
                            current_size, new_size
                        );
                        current_size = new_size;
                        let _ = tx.start_send(current_size).unwrap();
                        let _ = tx.poll_complete().unwrap();
                    },
                    Err(e) => {
                        error!("failed to get terminal size: {}", e);
                    }
                }
                sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stdout(&mut self) -> &mut MouseTerminal<AlternateScreen<RawTerminal<Stdout>>> {
        &mut self.stdout
    }
}

pub enum TerminalEvent {
    Resize((u16, u16)),
    Input(Event),
}

impl Stream for Terminal {
    type Item = TerminalEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.size.poll() {
            Ok(Async::Ready(Some(size))) => {
                return Ok(Async::Ready(Some(TerminalEvent::Resize(size))))
            }
            Ok(Async::Ready(None)) => {
                warn!("terminal size sender closed the channel");
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {}
            Err(()) => return Err(()),
        }
        match self.stdin.poll() {
            Ok(Async::Ready(Some(event))) => {
                return Ok(Async::Ready(Some(TerminalEvent::Input(event))))
            }
            Ok(Async::Ready(None)) => {
                warn!("terminal input sender closed the channel");
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {}
            Err(()) => return Err(()),
        }
        Ok(Async::NotReady)
    }
}
