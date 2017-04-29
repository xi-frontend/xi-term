use std::process::ChildStdin;
use std::collections::HashMap;

use xi_rpc::{Handler, RpcCtx};
use screen::Screen;
use input::{self, Input};

use serde_json::Value;
use view::View;
use client::Client;


pub struct Core {
    pub client: Client,
    views: HashMap<String, View>,
    view_id: Option<String>,
    input: Input,
    screen: Screen,
}

impl Handler<ChildStdin> for Core {
    fn handle_notification(&mut self, ctx: RpcCtx<ChildStdin>, method: &str, params: &Value) {
        info!("handling notification (method: {}, params: {:?})", method, params);
        // TODO
    }

    fn handle_request(&mut self, ctx: RpcCtx<ChildStdin>, method: &str, params: &Value) -> Result<Value, Value> {
        info!("handling request");
        // TODO
        Ok(json!([]))
    }

    fn idle(&mut self, ctx: RpcCtx<ChildStdin>, _token: usize) {
        info!("idling");
        if let Ok(event) = self.input.try_recv() {
            input::handle(&event, self);
        }
    }
}

impl Core {
    pub fn new(client: Client) -> Self {
        Core {
            client: client,
            screen: Screen::new(),
            input: Input::new(),
            views: HashMap::new(),
            view_id: None,
        }
    }

    fn get_view(&self) -> &str {
        if let Some(ref id) = self.view_id {
            id
        } else {
            ""
        }
    }

    pub fn new_view(&self, path: Option<&str>) -> Result<String, ()> {
        self.client.new_view(path)
    }

    pub fn close_view(&self) {
        self.client.close_view(self.get_view())
    }

    pub fn save(&self, file_path: &str) {
        self.client.save(self.get_view(), file_path);
    }

    pub fn insert(&self, chars: &str) {
        self.client.insert(self.get_view(), chars);
    }

    pub fn scroll(&self, start: u64, end: u64) {
        unimplemented!()
    }

    pub fn click(&self, line: u64, column: u64) {
        unimplemented!()
    }

    pub fn drag(&self, line: u64, column: u64) {
        unimplemented!()
    }

    pub fn delete_backward(&self) {
        unimplemented!()
    }

    pub fn insert_newline(&self) {
        unimplemented!()
    }

    pub fn move_up(&self) {
        unimplemented!()
    }

    pub fn move_up_and_modify_selection(&self) {
        unimplemented!()
    }

    pub fn move_down(&self) {
        unimplemented!()
    }

    pub fn move_down_and_modify_selection(&self) {
        unimplemented!()
    }

    pub fn move_left(&self) {
        unimplemented!()
    }

    pub fn move_left_and_modify_selection(&self) {
        unimplemented!()
    }

    pub fn move_right(&self) {
        unimplemented!()
    }

    pub fn move_right_and_modify_selection(&self) {
        unimplemented!()
    }

    pub fn scroll_page_up(&self) {
        unimplemented!()
    }

    pub fn page_up(&self) {
        unimplemented!()
    }

    pub fn page_up_and_modify_selection(&self) {
        unimplemented!()
    }

    pub fn scroll_page_down(&self) {
        unimplemented!()
    }

    pub fn page_down(&self) {
        unimplemented!()
    }

    pub fn page_down_and_modify_selection(&self) {
        unimplemented!()
    }
}

