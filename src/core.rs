use std::process::ChildStdin;
use std::collections::HashMap;

use xi_rpc::{self, Handler, RpcPeer, RpcCtx};
use screen::Screen;
use input::{self, Input};

use serde_json::Value;
use view::View;


pub struct Core {
    peer: RpcPeer<ChildStdin>,
    views: HashMap<String, View>,
    view_id: Option<String>,
    input: Input,
    screen: Screen,
}

impl Handler<ChildStdin> for Core {
    fn handle_notification(&mut self, ctx: RpcCtx<ChildStdin>, method: &str, params: &Value) {
        unimplemented!()
    }

    fn handle_request(&mut self,
                      ctx: RpcCtx<ChildStdin>,
                      method: &str,
                      params: &Value)
                      -> Result<Value, Value> {
        unimplemented!()
    }

    fn idle(&mut self, ctx: RpcCtx<ChildStdin>, _token: usize) {
        if let Ok(event) = self.input.try_recv() {
            input::handle(&event, self);
        }
    }
}

impl Core {
    pub fn new(peer: RpcPeer<ChildStdin>) -> Self {
        Core {
            peer: peer,
            screen: Screen::new(),
            input: Input::new(),
            views: HashMap::new(),
            view_id: None,
        }
    }

    fn request(&self, method: &str, params: &Value) -> Result<Value, xi_rpc::Error> {
        self.peer.send_rpc_request(method, params)
    }

    fn notify(&self, method: &str, params: &Value) {
        self.peer.send_rpc_notification(method, params);
    }

    pub fn new_view(&self, path: Option<String>) -> Result<String, ()> {
        self.request("new_view", &json!({"file_path": path}))
            .map(|value| value.as_str().unwrap().to_owned())
            .map_err(|_| unimplemented!())
    }

    pub fn close_view(&self) {
        self.notify("close_view", &json!({"view_id": self.view_id.unwrap()}))
    }

    pub fn save(&self, file_path: &str) {
        self.notify("save", &json!({
            "file_path": file_path,
            "view_id": self.view_id.unwrap(),
        }))
    }

    fn edit(&self, method: &str, params: Option<&Value>) {
        let msg = json!({
            "method": method,
            "view_id": self.view_id.unwrap(),
            "params": params.unwrap_or_else(|| &Value::Array(vec![])),
        });
        self.notify("edit", &msg);
    }

    pub fn insert(&self, chars: &str) {
        self.edit("insert", Some(&json!({"chars": chars})))
    }

    pub fn scroll(&self, start: u64, end: u64) {
        self.edit("scroll", Some(&json!([start, end])))
    }

    pub fn click(&self, line: u64, column: u64) {
        // [line, col, modifier, click count]
        // for now we only handle single left click
        self.edit("click", Some(&json!([line, column, 0, 1])));
    }

    pub fn drag(&self, line: u64, column: u64) {
        // [line, col, modifier]
        // for now we only handle left click
        self.edit("drag", Some(&json!([line, column, 0])));
    }

    pub fn delete_backward(&self) {
        self.edit("delete_backward", None)
    }

    pub fn insert_newline(&self) {
        self.edit("insert_newline", None)
    }

    pub fn move_up(&self) {
        self.edit("move_up", None)
    }

    pub fn move_up_and_modify_selection(&self) {
        self.edit("move_up_and_modify_selection", None)
    }

    pub fn move_down(&self) {
        self.edit("move_down", None)
    }

    pub fn move_down_and_modify_selection(&self) {
        self.edit("move_down_and_modify_selection", None)
    }

    pub fn move_left(&self) {
        self.edit("move_left", None)
    }

    pub fn move_left_and_modify_selection(&self) {
        self.edit("move_left_and_modify_selection", None)
    }

    pub fn move_right(&self) {
        self.edit("move_right", None)
    }

    pub fn move_right_and_modify_selection(&self) {
        self.edit("move_right_and_modify_selection", None)
    }

    pub fn scroll_page_up(&self) {
        self.edit("scroll_page_up", None)
    }

    pub fn page_up(&self) {
        self.edit("page_up", None)
    }

    pub fn page_up_and_modify_selection(&self) {
        self.edit("page_up_and_modify_selection", None)
    }

    pub fn scroll_page_down(&self) {
        self.edit("scroll_page_down", None)
    }

    pub fn page_down(&self) {
        self.edit("page_down", None)
    }

    pub fn page_down_and_modify_selection(&self) {
        self.edit("page_down_and_modify_selection", None)
    }
}
