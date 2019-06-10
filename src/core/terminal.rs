use std::io::{self, Stdout};
use std::thread::{sleep, spawn};
use std::time::Duration;

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{Async, Poll, Sink, Stream};

use failure::{Error, ResultExt};

use termion::event::Event;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;

/// Simple type alias for the Write implementer we render to.
pub type RenderTarget = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

pub struct Terminal {
    size: UnboundedReceiver<(u16, u16)>,
    stdin: UnboundedReceiver<Event>,
    stdout: RenderTarget,
}

impl Terminal {
    pub fn new() -> Result<Self, Error> {
        let (stdin_tx, stdin_rx) = unbounded();
        let (size_tx, size_rx) = unbounded();
        let stdout = MouseTerminal::from(AlternateScreen::from(
            io::stdout()
                .into_raw_mode()
                .context("Failed to put terminal into raw mode")?,
        ));

        let term = Terminal {
            stdin: stdin_rx,
            size: size_rx,
            stdout,
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
                    Ok(new_size) => {
                        if new_size != current_size {
                            info!(
                                "terminal resized (from {:?} to {:?})",
                                current_size, new_size
                            );
                            current_size = new_size;
                            let _ = tx.start_send(current_size).unwrap();
                            let _ = tx.poll_complete().unwrap();
                        }
                    }
                    Err(e) => {
                        error!("failed to get terminal size: {}", e);
                    }
                }
                sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stdout(&mut self) -> &mut RenderTarget {
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
        debug!("polling for terminal size events");
        match self.size.poll() {
            Ok(Async::Ready(Some(size))) => {
                debug!("size event: {:?}", size);
                let event = TerminalEvent::Resize(size);
                return Ok(Async::Ready(Some(event)));
            }
            Ok(Async::Ready(None)) => {
                warn!("terminal size sender closed the channel");
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                debug!("done polling for terminal size events");
            }
            Err(()) => return Err(()),
        }

        debug!("polling for stdin events");
        match self.stdin.poll() {
            Ok(Async::Ready(Some(event))) => {
                debug!("stdin event: {:?}", event);
                let event = TerminalEvent::Input(event);
                return Ok(Async::Ready(Some(event)));
            }
            Ok(Async::Ready(None)) => {
                warn!("terminal input sender closed the channel");
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {
                debug!("done polling for stdin events");
            }
            Err(()) => return Err(()),
        }
        debug!("done polling the terminal");
        Ok(Async::NotReady)
    }
}
