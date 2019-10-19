use std::io::{self, Write};

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::sync::oneshot::{self, Receiver, Sender};
use futures::{Async, Future, Poll, Sink, Stream};

use termion::event::{Event, Key};
use xrl::{Client, Frontend, FrontendBuilder, MeasureWidth, XiNotification};

use failure::Error;

use core::{Command, Terminal, TerminalEvent, Settings};
use widgets::{CommandPrompt, Editor};

pub struct Tui {
    /// The editor holds the text buffers (named "views" in xi
    /// terminology).
    editor: Editor,

    /// The application settings, read in at startup
    settings: Settings,

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
    pub fn new(client: Client,
               events: UnboundedReceiver<CoreEvent>,
               settings: Settings
    ) -> Result<Self, Error> {
        println!("{:?}", settings);
        Ok(Tui {
            terminal: Terminal::new()?,
            settings: settings,
            exit: false,
            term_size: (0, 0),
            editor: Editor::new(client),
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
            Command::Cancel => {
                self.prompt = None;
            }
            Command::Quit => self.exit = true,
            Command::Save(view) => self.editor.save(view),
            Command::Back => self.editor.back(),
            Command::Delete => self.editor.delete(),
            Command::Open(file) => self.editor.new_view(file),
            Command::SetTheme(theme) => self.editor.set_theme(&theme),
            Command::NextBuffer => self.editor.next_buffer(),
            Command::PrevBuffer => self.editor.prev_buffer(),
            Command::MoveLeft => self.editor.move_left(),
            Command::MoveRight => self.editor.move_right(),
            Command::MoveUp => self.editor.move_up(),
            Command::MoveDown => self.editor.move_down(),
            Command::PageDown => self.editor.page_down(),
            Command::PageUp => self.editor.page_up(),
            Command::ToggleLineNumbers => self.editor.toggle_line_numbers(),
            Command::Noop => {},
        }
    }

    /// Global keybindings can be parsed here
    fn handle_input(&mut self, event: Event) {
        debug!("handling input {:?}", event);
        // TODO handle input events based on settings here
        //self.run_command(self.settings.get_command(event));


        // Get the command for this key event based on the override layer
        // if (let command = get_command(Event, "override") ==

        match event {
            Event::Key(Key::Ctrl('c')) => self.exit = true,
            Event::Key(Key::Alt('x')) => {
                if let Some(ref mut prompt) = self.prompt {
                    match prompt.handle_input(&event) {
                        Ok(None) => {}
                        Ok(Some(_)) => unreachable!(),
                        Err(_) => unreachable!(),
                    }
                } else {
                    self.prompt = Some(CommandPrompt::default());
                }
            }
            event => {
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
