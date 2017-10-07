use std::io::{self, Write};
use std::collections::HashMap;

use futures::{future, Async, Future, Poll, Sink, Stream};
use tokio_core::reactor::Handle;

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

use termion::event::{Event, Key, MouseButton, MouseEvent};
use xrl::{Client, ClientResult, Frontend, FrontendBuilder, ScrollTo, ServerResult, Style, Update};

use errors::*;
use terminal::{Terminal, TerminalEvent};
use view::View;

pub struct Tui {
    pub pending_open_requests: Vec<ClientResult<String>>,
    pub delayed_events: Vec<CoreEvent>,
    pub views: HashMap<String, View>,
    pub current_view: String,
    pub events: UnboundedReceiver<CoreEvent>,
    pub handle: Handle,
    pub client: Client,
    pub term: Terminal,
    pub term_size: (u16, u16),
    pub shutdown: bool,
    pub styles: HashMap<u64, Style>,
}

impl Tui {
    pub fn new(
        handle: Handle,
        client: Client,
        events: UnboundedReceiver<CoreEvent>,
    ) -> Result<Self> {
        let mut styles = HashMap::new();
        styles.insert(0, Default::default());

        Ok(Tui {
            events: events,
            delayed_events: Vec::new(),
            pending_open_requests: Vec::new(),
            handle: handle,
            term: Terminal::new()?,
            term_size: (0, 0),
            views: HashMap::new(),
            styles: styles,
            current_view: "".into(),
            client: client,
            shutdown: false,
        })
    }

    pub fn handle_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Update(update) => self.handle_update(update),
            CoreEvent::SetStyle(style) => self.handle_set_style(style),
            CoreEvent::ScrollTo(scroll_to) => self.handle_scroll_to(scroll_to),
        }
    }

    pub fn handle_update(&mut self, update: Update) {
        let Tui {
            ref mut views,
            ref mut delayed_events,
            ..
        } = *self;
        match views.get_mut(&update.view_id) {
            Some(view) => view.update_cache(update),
            None => delayed_events.push(CoreEvent::Update(update)),
        }
    }

    pub fn handle_scroll_to(&mut self, scroll_to: ScrollTo) {
        let Tui {
            ref mut views,
            ref mut delayed_events,
            ..
        } = *self;
        match views.get_mut(&scroll_to.view_id) {
            Some(view) => view.set_cursor(scroll_to.line, scroll_to.column),
            None => delayed_events.push(CoreEvent::ScrollTo(scroll_to)),
        }
    }

    pub fn handle_set_style(&mut self, style: Style) {
        self.styles.insert(style.id, style);
    }

    pub fn handle_resize(&mut self, size: (u16, u16)) {
        info!("setting new terminal size");
        self.term_size = size;
        let future = self.client
            .scroll(&self.current_view, u64::from(size.1), u64::from(size.0))
            .map_err(|_| ());
        self.handle.spawn(future);
    }

    pub fn insert(&mut self, character: char) {
        let future = self.client
            .char(&self.current_view, character)
            .map_err(|_| ());
        self.handle.spawn(future);
    }

    fn down(&mut self) {
        let future = self.client.down(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn up(&mut self) {
        let future = self.client.up(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn left(&mut self) {
        let future = self.client.left(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn right(&mut self) {
        let future = self.client.right(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn page_down(&mut self) {
        let future = self.client.page_down(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn page_up(&mut self) {
        let future = self.client.page_up(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn delete(&mut self) {
        let future = self.client.del(&self.current_view).map_err(|_| ());
        self.handle.spawn(future);
    }

    pub fn open(&mut self, file_path: &str) {
        let task = self.client.new_view(Some(file_path.to_string()));
        self.pending_open_requests.push(task);
    }

    pub fn exit(&mut self) {
        self.shutdown = true;
    }

    pub fn save(&mut self) {
        unimplemented!()
    }

    pub fn click(&mut self, x: u64, y: u64) {
        let Tui {
            ref mut client,
            ref mut views,
            ref mut handle,
            ref current_view,
            ..
        } = *self;
        if let Some(view) = views.get_mut(current_view) {
            let (line, column) = view.click(x, y);
            let future = client.click(current_view, line, column).map_err(|_| ());
            handle.spawn(future);
        }
        error!("view not found");
    }

    pub fn drag(&mut self, x: u64, y: u64) {
        let Tui {
            ref mut client,
            ref mut views,
            ref mut handle,
            ref current_view,
            ..
        } = *self;
        if let Some(view) = views.get_mut(current_view) {
            let (line, column) = view.click(x, y);
            let future = client.drag(current_view, line, column).map_err(|_| ());
            handle.spawn(future);
        }
        error!("view not found");
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Char(c) => self.insert(c),
                Key::Ctrl(c) => match c {
                    'c' => self.exit(),
                    'w' => self.save(),
                    _ => error!("un-handled input ctrl+{}", c),
                },
                Key::Backspace => self.delete(),
                Key::Left => self.left(),
                Key::Right => self.right(),
                Key::Up => self.up(),
                Key::Down => self.down(),
                Key::PageUp => self.page_up(),
                Key::PageDown => self.page_down(),
                k => error!("un-handled key {:?}", k),
            },
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Press(press_event, y, x) => match press_event {
                    MouseButton::Left => self.click(u64::from(x) - 1, u64::from(y) - 1),
                    MouseButton::WheelUp => self.up(),
                    MouseButton::WheelDown => self.down(),
                    button => error!("un-handled button {:?}", button),
                },
                MouseEvent::Release(..) => {}
                MouseEvent::Hold(y, x) => self.drag(u64::from(x) - 1, u64::from(y) - 1),
            },
            ev => error!("un-handled event {:?}", ev),
        }
    }

    pub fn process_open_requests(&mut self) {
        if self.pending_open_requests.is_empty() {
            return;
        }

        info!("process pending open requests");

        let Tui {
            ref mut pending_open_requests,
            ref mut views,
            ref mut current_view,
            ref mut client,
            ref mut handle,
            ref term_size,
            ..
        } = *self;

        let mut done = vec![];
        for (idx, task) in pending_open_requests.iter_mut().enumerate() {
            match task.poll() {
                Ok(Async::Ready(view_id)) => {
                    info!("open request succeeded for {}", &view_id);
                    done.push(idx);
                    views.insert(view_id.clone(), View::new());
                    *current_view = view_id;

                    info!("notifying the core about the scroll region for the view");
                    let future = client
                        .scroll(current_view, u64::from(term_size.1), u64::from(term_size.0))
                        .map_err(|_| ());
                    handle.spawn(future);
                }
                Ok(Async::NotReady) => continue,
                Err(e) => panic!("\"open\" task failed: {}", e),
            }
        }
        for idx in done.iter().rev() {
            pending_open_requests.remove(*idx);
        }

        if pending_open_requests.is_empty() {
            info!("no more pending open request");
        }
    }

    pub fn process_terminal_events(&mut self) {
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

    pub fn process_core_events(&mut self) {
        loop {
            match self.events.poll() {
                Ok(Async::Ready(Some(event))) => {
                    self.handle_core_event(event);
                }
                Ok(Async::Ready(None)) => {
                    error!("core stdout shut down => panicking");
                    panic!("core stdout shut down");
                }
                Ok(Async::NotReady) => break,
                Err(_) => {
                    error!("error while polling core => panicking");
                    panic!("error while polling core");
                }
            }
        }
    }

    pub fn process_delayed_events(&mut self) {
        let delayed_events: Vec<CoreEvent> = self.delayed_events.drain(..).collect();
        for event in delayed_events {
            self.handle_core_event(event);
        }
    }

    pub fn render(&mut self) -> Result<()> {
        let Tui {
            ref mut views,
            ref mut term,
            ref current_view,
            ref term_size,
            ref styles,
            ..
        } = *self;
        if let Some(view) = views.get_mut(current_view) {
            view.resize(term_size.1);
            view.render(term.stdout(), styles)?;
            if let Err(e) = term.stdout().flush() {
                error!("failed to flush stdout: {}", e);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum CoreEvent {
    Update(Update),
    ScrollTo(ScrollTo),
    SetStyle(Style),
}

impl Future for Tui {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.process_open_requests();
        self.process_delayed_events();
        self.process_terminal_events();
        self.process_core_events();

        if let Err(e) = self.render() {
            log_error(&e);
        }

        if self.shutdown {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

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

    fn set_style(&mut self, style: Style) -> ServerResult<()> {
        self.send_core_event(CoreEvent::SetStyle(style))
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
