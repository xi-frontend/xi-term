use std::io::{self, Write};

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{future, Async, Future, Poll, Sink, Stream};

use termion::event::{Event, Key};
use tokio::run;
use xrl::{
    AvailablePlugins, Client, ConfigChanged, Frontend, FrontendBuilder, PluginStarted,
    PluginStoped, ScrollTo, ServerResult, Style, ThemeChanged, Update, UpdateCmds,
};

use failure::Error;
use xdg::BaseDirectories;

use core::{Command, Terminal, TerminalEvent};
use widgets::{CommandPrompt, Editor};

pub struct Tui {
    editor: Editor,
    prompt: Option<CommandPrompt>,
    term: Terminal,
    term_size: (u16, u16),
    shutdown: bool,
}

impl Tui {
    pub fn new(mut client: Client, events: UnboundedReceiver<CoreEvent>) -> Result<Self, Error> {
        let conf_dir = BaseDirectories::with_prefix("xi")
            .ok()
            .and_then(|dirs| Some(dirs.get_config_home().to_string_lossy().into_owned()));
        run(client
            .client_started(conf_dir.as_ref().map(|dir| &**dir), None)
            .map_err(|_| ())
        );

        Ok(Tui {
            term: Terminal::new()?,
            shutdown: false,
            term_size: (0, 0),
            editor: Editor::new(client, events),
            prompt: None,
        })
    }

    fn handle_resize(&mut self, size: (u16, u16)) {
        self.term_size = size;
        self.editor.handle_resize(size);
    }

    fn exit(&mut self) {
        self.shutdown = true;
    }

    pub fn handle_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::Cancel => {
                self.prompt = None;
            }
            Command::Quit => self.exit(),
            Command::Save(view) => self.editor.save(view),
            Command::Back => self.editor.back(),
            Command::Delete => self.editor.delete(),
            Command::Open(file) => self.editor.open(file),
            Command::SetTheme(theme) => self.editor.set_theme(&theme),
            Command::NextBuffer => self.editor.next_buffer(),
            Command::PrevBuffer => self.editor.prev_buffer(),
            Command::PageDown => self.editor.page_down(),
            Command::PageUp => self.editor.page_up(),
        }
    }

    /// Global keybindings can be parsed here
    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(Key::Ctrl('c')) => self.exit(),
            Event::Key(Key::Alt('x')) => {
                if let Some(ref mut prompt) = self.prompt {
                    match prompt.handle_input(&event) {
                        Ok(None) => {},
                        Ok(Some(_)) => unreachable!(),
                        Err(_) => unreachable!()
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
                    Ok(None) => { self.prompt = Some(prompt); },
                    Ok(Some(cmd)) => self.handle_cmd(cmd),
                    Err(err) => {
                        error!("Failed to parse command: {:?}", err);
                    }
                }
            }
        }
    }

    /// Check and handle terminal events.
    fn process_terminal_events(&mut self) {
        let mut new_size: Option<(u16, u16)> = None;
        loop {
            match self.term.poll() {
                Ok(Async::Ready(Some(event))) => match event {
                    TerminalEvent::Resize(size) => {
                        new_size = Some(size);
                    }
                    TerminalEvent::Input(input) => self.handle_input(input),
                },
                Ok(Async::Ready(None)) => {
                    error!("terminal stream shut down => exiting");
                    self.shutdown = true;
                }
                Ok(Async::NotReady) => break,
                Err(_) => {
                    error!("error while polling terminal stream => exiting");
                    self.shutdown = true;
                }
            }
        }
        if let Some(size) = new_size {
            if !self.shutdown {
                self.handle_resize(size);
            }
        }
    }

    fn render(&mut self) -> Result<(), Error> {
        if let Some(ref mut prompt) = self.prompt {
            prompt.render(self.term.stdout(), self.term_size.1)?;
        } else {
            self.editor.render(self.term.stdout())?;
        }
        if let Err(e) = self.term.stdout().flush() {
            error!("failed to flush stdout: {}", e);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum CoreEvent {
    Update(Update),
    ScrollTo(ScrollTo),
    SetStyle(Style),
    ConfigChanged(ConfigChanged)
}

impl Future for Tui {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.editor.process_open_requests();
        self.editor.process_delayed_events();
        self.process_terminal_events();
        self.editor.process_core_events();

        if let Err(e) = self.render() {
            error!("error: {}", e);
            error!("caused by: {}", e.cause());
        }

        if self.shutdown {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

/// Actual frontend that implementes `xrl::Frontend`
/// It's future is the main loop of xi-term.
pub struct TuiService(UnboundedSender<CoreEvent>);

impl TuiService {
    fn send_core_event(&mut self, event: CoreEvent) -> ServerResult<()> {
        if let Err(e) = self.0.start_send(event) {
            let e = format!("failed to send core event to TUI: {}", e);
            error!("{}", e);
            return Box::new(future::err(e.into()));
        }
        match self.0.poll_complete() {
            Ok(_) => Box::new(future::ok(())),
            Err(e) => {
                let e = format!("failed to send core event to TUI: {}", e);
                Box::new(future::err(e.into()))
            }
        }
    }
}

impl Frontend for TuiService {
    fn update(&mut self, update: Update) -> ServerResult<()> {
        self.send_core_event(CoreEvent::Update(update))
    }

    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()> {
        self.send_core_event(CoreEvent::ScrollTo(scroll_to))
    }

    fn def_style(&mut self, style: Style) -> ServerResult<()> {
        self.send_core_event(CoreEvent::SetStyle(style))
    }
    fn available_plugins(&mut self, _plugins: AvailablePlugins) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn update_cmds(&mut self, _plugins: UpdateCmds) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn plugin_started(&mut self, _plugins: PluginStarted) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn plugin_stoped(&mut self, _plugin: PluginStoped) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
    fn config_changed(&mut self, config: ConfigChanged) -> ServerResult<()> {
        self.send_core_event(CoreEvent::ConfigChanged(config))
    }
    fn theme_changed(&mut self, _theme: ThemeChanged) -> ServerResult<()> {
        Box::new(future::ok(()))
    }
}

pub struct TuiServiceBuilder(UnboundedSender<CoreEvent>);

impl TuiServiceBuilder {
    pub fn new() -> (Self, UnboundedReceiver<CoreEvent>) {
        let (tx, rx) = unbounded();
        (TuiServiceBuilder(tx), rx)
    }
}

impl FrontendBuilder<TuiService> for TuiServiceBuilder {
    fn build(self, _client: Client) -> TuiService {
        TuiService(self.0)
    }
}
