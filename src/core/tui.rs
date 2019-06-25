use std::io::{self, Write};

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::sync::oneshot::{self, Receiver, Sender};
use futures::{Async, Future, Poll, Sink, Stream};

use termion::event::{Event};
use xrl::{Client, Frontend, FrontendBuilder, MeasureWidth, XiNotification};

use failure::Error;

use crate::core::{Command, Terminal, TerminalEvent, KeybindingConfig};
use crate::widgets::{CommandPrompt, Editor};

pub struct Tui {
    /// The editor holds the text buffers (named "views" in xi
    /// terminology).
    editor: Editor,

    /// The command prompt is where users can type commands.
    prompt: Option<CommandPrompt>,

    /// The terminal is used to draw on the screen a get inputs from
    /// the user.
    terminal: Terminal,

    /// The size of the terminal.
    term_size: (u16, u16),

    /// Whether the editor is shutting down.
    exit: bool,

    /// Stream of messages from Xi core.
    core_events: UnboundedReceiver<CoreEvent>,
}

impl Tui {
    /// Create a new Tui instance.
    pub fn new(client: Client, events: UnboundedReceiver<CoreEvent>, keybindings: KeybindingConfig) -> Result<Self, Error> {
        Ok(Tui {
            terminal: Terminal::new()?,
            exit: false,
            term_size: (0, 0),
            editor: Editor::new(client, keybindings),
            prompt: None,
            core_events: events,
        })
    }

    fn handle_resize(&mut self, size: (u16, u16)) {
        self.term_size = size;
        self.editor.handle_resize(size);
    }

    pub fn run_command(&mut self, cmd: Command) {
        match cmd {
            // We handle these here, the rest is the job of the editor
            Command::OpenPrompt => self.open_prompt(),
            Command::Cancel => self.prompt = None,
            Command::Quit => self.exit = true,

            editor_cmd => self.editor.handle_command(editor_cmd)
        }
    }

    fn open_prompt(&mut self) {
        if self.prompt.is_none() {
            self.prompt = Some(CommandPrompt::default());
        }
    }

    /// Global keybindings can be parsed here
    fn handle_input(&mut self, event: Event) {
        debug!("handling input {:?}", event);
        // TODO: Translate here to own enum which supports more event-types
        if let Some(cmd) = self.editor.keybindings.keymap.get(&event) {
            match cmd {
                Command::OpenPrompt => {
                                        if self.prompt.is_none() {
                                            self.prompt = Some(CommandPrompt::default());
                                        }
                                        return; },
                Command::Quit => { self.exit = true; return; },
                _ => {/* Somebody else has to deal with these commands */},
            }
        }

        // No command prompt is active, process the event normally.
        if self.prompt.is_none() {
            self.editor.handle_input(event);
            return;
        }

        // A command prompt is active.
        let mut prompt = self.prompt.take().unwrap();
        match prompt.handle_input(&event) {
            Ok(None) => {
                self.prompt = Some(prompt);
            }
            Ok(Some(cmd)) => self.run_command(cmd),
            Err(err) => {
                error!("Failed to parse command: {:?}", err);
            }
        }
    }

    fn render(&mut self) -> Result<(), Error> {
        if let Some(ref mut prompt) = self.prompt {
            prompt.render(self.terminal.stdout(), self.term_size.1)?;
        } else {
            self.editor.render(self.terminal.stdout())?;
        }
        if let Err(e) = self.terminal.stdout().flush() {
            error!("failed to flush stdout: {}", e);
        }
        Ok(())
    }

    fn handle_core_event(&mut self, event: CoreEvent) {
        self.editor.handle_core_event(event)
    }

    fn poll_editor(&mut self) {
        debug!("polling the editor");
        match self.editor.poll() {
            Ok(Async::NotReady) => {
                debug!("no more editor event, done polling");
                return;
            }
            Ok(Async::Ready(_)) => {
                info!("The editor exited normally. Shutting down the TUI");
                self.exit = true;
                return;
            }
            Err(e) => {
                error!("The editor exited with an error: {:?}", e);
                error!("Shutting down the TUI.");
                self.exit = true;
                return;
            }
        }
    }

    fn poll_terminal(&mut self) {
        debug!("polling the terminal");
        loop {
            match self.terminal.poll() {
                Ok(Async::Ready(Some(event))) => match event {
                    TerminalEvent::Input(event) => self.handle_input(event),
                    TerminalEvent::Resize(event) => self.handle_resize(event),
                },
                Ok(Async::Ready(None)) => {
                    info!("The terminal exited normally. Shutting down the TUI");
                    self.exit = true;
                    return;
                }
                Ok(Async::NotReady) => {
                    debug!("no more terminal event, done polling");
                    return;
                }
                Err(e) => {
                    error!("The terminal exited with an error: {:?}", e);
                    error!("Shutting down the TUI");
                    self.exit = true;
                    return;
                }
            }
        }
    }

    fn poll_rpc(&mut self) {
        debug!("polling for RPC messages");
        loop {
            match self.core_events.poll() {
                Ok(Async::Ready(Some(event))) => self.handle_core_event(event),
                Ok(Async::Ready(None)) => {
                    info!("The RPC endpoint exited normally. Shutting down the TUI");
                    self.exit = true;
                    return;
                }
                Ok(Async::NotReady) => {
                    debug!("no more RPC event, done polling");
                    return;
                }
                Err(e) => {
                    error!("The RPC endpoint exited with an error: {:?}", e);
                    error!("Shutting down the TUI");
                    self.exit = true;
                    return;
                }
            }
        }
    }
}

impl Future for Tui {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("polling the TUI");
        self.poll_terminal();
        if self.exit {
            info!("exiting the TUI");
            return Ok(Async::Ready(()));
        }

        self.poll_editor();
        if self.exit {
            info!("exiting the TUI");
            return Ok(Async::Ready(()));
        }

        self.poll_rpc();
        if self.exit {
            info!("exiting the TUI");
            return Ok(Async::Ready(()));
        }

        debug!("done polling the TUI components");
        debug!("rendering");
        self.render().expect("failed to render the TUI");
        debug!("done rendering, end of polling");
        Ok(Async::NotReady)
    }
}

pub enum CoreEvent {
    Notify(XiNotification),
    MeasureWidth((MeasureWidth, Sender<Vec<Vec<f32>>>)),
}

pub struct TuiService(UnboundedSender<CoreEvent>);

impl Frontend for TuiService {
    type NotificationResult = Result<(), ()>;
    fn handle_notification(&mut self, notification: XiNotification) -> Self::NotificationResult {
        self.0.start_send(CoreEvent::Notify(notification)).unwrap();
        self.0.poll_complete().unwrap();
        Ok(())
    }

    type MeasureWidthResult = NoErrorReceiver<Vec<Vec<f32>>>;
    fn handle_measure_width(&mut self, request: MeasureWidth) -> Self::MeasureWidthResult {
        let (tx, rx) = oneshot::channel::<Vec<Vec<f32>>>();
        self.0
            .start_send(CoreEvent::MeasureWidth((request, tx)))
            .unwrap();
        self.0.poll_complete().unwrap();
        NoErrorReceiver(rx)
    }
}

/// A dummy type from wrap a `oneshot::Receiver`.
///
/// The only difference with the `oneshot::Receiver` is that
/// `NoErrorReceiver`'s future implementation uses the empty type `()`
/// for its error.
pub struct NoErrorReceiver<T>(Receiver<T>);

impl<T> Future for NoErrorReceiver<T> {
    type Item = T;
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll().map_err(|_cancelled| ())
    }
}

pub struct TuiServiceBuilder(UnboundedSender<CoreEvent>);

impl TuiServiceBuilder {
    pub fn new() -> (Self, UnboundedReceiver<CoreEvent>) {
        let (tx, rx) = unbounded();
        (TuiServiceBuilder(tx), rx)
    }
}

impl FrontendBuilder for TuiServiceBuilder {
    type Frontend = TuiService;
    fn build(self, _client: Client) -> Self::Frontend {
        TuiService(self.0)
    }
}
