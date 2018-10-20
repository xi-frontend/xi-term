use std::collections::HashMap;
use std::io::Write;

use futures::sync::mpsc::UnboundedReceiver;
use futures::{Async, Future, Stream};

use termion::event::Event;
use tokio::run;
use xrl::{Client, ClientResult, ScrollTo, Style, Update, ViewId};
use indexmap::IndexMap;
use failure::Error;

use core::CoreEvent;
use widgets::{View, ViewClient};

/// The main interface to xi-core
pub struct Editor {
    pub pending_open_requests: Vec<ClientResult<(ViewId, View)>>,
    pub delayed_events: Vec<CoreEvent>,
    pub views: IndexMap<ViewId, View>,
    pub current_view: ViewId,
    pub events: UnboundedReceiver<CoreEvent>,
    pub client: Client,
    pub size: (u16, u16),
    pub styles: HashMap<u64, Style>,
}

/// Methods for general use.
impl Editor {
    pub fn new(client: Client, events: UnboundedReceiver<CoreEvent>) -> Editor {
        let mut styles = HashMap::new();
        styles.insert(0, Default::default());

        Editor {
            events,
            delayed_events: Vec::new(),
            pending_open_requests: Vec::new(),
            size: (0, 0),
            views: IndexMap::new(),
            styles,
            current_view: ViewId(0),
            client,
        }
    }
}

/// Methods related to terminal input.
impl Editor {
    pub fn handle_input(&mut self, event: Event) {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.handle_input(event)
        }
    }

    pub fn handle_resize(&mut self, size: (u16, u16)) {
        info!("setting new terminal size");
        self.size = size;
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.resize(size.1);
        } else {
            warn!("view {} not found", self.current_view);
        }
    }
}

/// Methods related to handling things received from xi-core.
impl Editor {
    pub fn dispatch_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Update(update) => self.handle_update(update),
            CoreEvent::SetStyle(style) => self.handle_def_style(style),
            CoreEvent::ScrollTo(scroll_to) => self.handle_scroll_to(scroll_to),
        }
    }

    fn handle_update(&mut self, update: Update) {
        match self.views.get_mut(&update.view_id) {
            Some(view) => view.update_cache(update),
            None => self.delayed_events.push(CoreEvent::Update(update)),
        }
    }

    fn handle_scroll_to(&mut self, scroll_to: ScrollTo) {
        match self.views.get_mut(&scroll_to.view_id) {
            Some(view) => view.set_cursor(scroll_to.line, scroll_to.column),
            None => self.delayed_events.push(CoreEvent::ScrollTo(scroll_to)),
        }
    }

    fn handle_def_style(&mut self, style: Style) {
        self.styles.insert(style.id, style);
    }
}

/// Methods related to sending xi requests.
impl Editor {
    pub fn open(&mut self, file_path: Option<String>) {
        let client = self.client.clone();
        let task = self
            .client
            .new_view(file_path.clone())
            .and_then(move |view_id| {
                let view_client = ViewClient::new(client, view_id);
                Ok((view_id, View::new(view_client, Some(file_path.unwrap_or_else(|| "".into())))))
            });
        self.pending_open_requests.push(Box::new(task));
    }

    pub fn set_theme(&mut self, theme: &str) {
        let future = self.client.set_theme(theme).map_err(|_| ());
        run(future);
    }

    pub fn save(&mut self, view: Option<ViewId>) {
        match view {
            Some(view_id) => {
                if let Some(view) = self.views.get_mut(&view_id) {
                    view.save();
                }
            }
            None => {
                if let Some(view) = self.views.get_mut(&self.current_view) {
                    view.save();
                }
            }
        }
    }

    pub fn back(&mut self) {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.back();
        }
    }

    pub fn delete(&mut self) {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.delete();
        }
    }

    pub fn next_buffer(&mut self) {
        if let Some((dex, _, _)) = self.views.get_full(&self.current_view) {
            if dex+1 == self.views.len() {
                if let Some((view, _)) = self.views.get_index(0) {
                    self.current_view = *view;
                }
            } else if let Some((view, _)) = self.views.get_index(dex+1) {
                    self.current_view = *view;
            }
        }
    }

    pub fn prev_buffer(&mut self) {
        if let Some((dex, _, _)) = self.views.get_full(&self.current_view) {
            if dex == 0 {
                if let Some((view, _)) = self.views.get_index(self.views.len()-1) {
                    self.current_view = *view;
                }
            } else if let Some((view, _)) = self.views.get_index(dex-1) {
                    self.current_view = *view;
            }
        }
    }
}

/// Methods ment to be called by the tui struct
impl Editor {
    pub fn process_open_requests(&mut self) {
        if self.pending_open_requests.is_empty() {
            return;
        }

        info!("process pending open requests");

        let mut done = vec![];
        for (idx, task) in self.pending_open_requests.iter_mut().enumerate() {
            match task.poll() {
                Ok(Async::Ready((id, mut view))) => {
                    info!("open request succeeded for {}", &id);
                    done.push(idx);
                    view.resize(self.size.1);
                    self.views.insert(id, view);
                    self.current_view = id;
                }
                Ok(Async::NotReady) => continue,
                Err(e) => panic!("\"open\" task failed: {}", e),
            }
        }
        for idx in done.iter().rev() {
            self.pending_open_requests.remove(*idx);
        }

        if self.pending_open_requests.is_empty() {
            info!("no more pending open request");
        }
    }

    pub fn process_core_events(&mut self) {
        loop {
            match self.events.poll() {
                Ok(Async::Ready(Some(event))) => {
                    self.dispatch_core_event(event);
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
            self.dispatch_core_event(event);
        }
    }

    pub fn render<W: Write>(&mut self, term: &mut W) -> Result<(), Error> {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.render(term, &self.styles)?;
        }
        Ok(())
    }
}
