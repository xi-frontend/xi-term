use std::process::ChildStdin;
use xi_rpc::{Error, RpcPeer};
use serde_json::{self, Value};



#[derive(Clone)]
pub struct Client(RpcPeer<ChildStdin>);


impl Client {
    pub fn new(peer: RpcPeer<ChildStdin>) -> Self {
        Client(peer)
    }

    fn request(&self, method: &str, params: &Value) -> Result<Value, Error> {
        self.0.send_rpc_request(method, params)
    }

    fn notify(&self, method: &str, params: &Value) {
        self.0.send_rpc_notification(method, params);
    }

    pub fn new_view(&self, path: Option<&str>) -> Result<String, ()> {
        self.request("new_view", &json!({"file_path": path}))
            .map(|value| value.as_str().unwrap().to_owned())
            .map_err(|_| unimplemented!())
    }

    pub fn close_view(&self, view_id: &str) {
        self.notify("close_view", &json!({"view_id": view_id}));
    }

    pub fn save(&self, view_id: &str, file_path: &str) {
        self.notify("save", &json!({
            "file_path": file_path,
            "view_id": view_id,
        }))
    }

    fn edit(&self, method: &str, view_id: &str, params: Option<&Value>) {
        // FIXME: this is ugly
        let empty_params = Value::Array(vec![]);
        let msg = json!({
            "method": method,
            "view_id": view_id,
            "params": params.unwrap_or_else(|| &empty_params),
        });
        self.notify("edit", &msg);
    }

    pub fn insert(&self, view_id: &str, chars: &str) {
        self.edit("insert", view_id, Some(&json!({"chars": chars})))
    }

    pub fn scroll(&self, view_id: &str, start: u64, end: u64) {
        self.edit("scroll", view_id, Some(&json!([start, end])))
    }

    pub fn click(&self, view_id: &str, line: u64, column: u64) {
        // [line, col, modifier, click count]
        // for now we only handle single left click
        self.edit("click", view_id, Some(&json!([line, column, 0, 1])));
    }

    pub fn drag(&self, view_id: &str, line: u64, column: u64) {
        // [line, col, modifier]
        // for now we only handle left click
        self.edit("drag", view_id, Some(&json!([line, column, 0])));
    }

    pub fn delete_backward(&self, view_id: &str) {
        self.edit("delete_backward", view_id, None)
    }

    pub fn insert_newline(&self, view_id: &str) {
        self.edit("insert_newline", view_id, None)
    }

    pub fn move_up(&self, view_id: &str) {
        self.edit("move_up", view_id, None)
    }

    pub fn move_up_and_modify_selection(&self, view_id: &str) {
        self.edit("move_up_and_modify_selection", view_id, None)
    }

    pub fn move_down(&self, view_id: &str) {
        self.edit("move_down", view_id, None)
    }

    pub fn move_down_and_modify_selection(&self, view_id: &str) {
        self.edit("move_down_and_modify_selection", view_id, None)
    }

    pub fn move_left(&self, view_id: &str) {
        self.edit("move_left", view_id, None)
    }

    pub fn move_left_and_modify_selection(&self, view_id: &str) {
        self.edit("move_left_and_modify_selection", view_id, None)
    }

    pub fn move_right(&self, view_id: &str) {
        self.edit("move_right", view_id, None)
    }

    pub fn move_right_and_modify_selection(&self, view_id: &str) {
        self.edit("move_right_and_modify_selection", view_id, None)
    }

    pub fn scroll_page_up(&self, view_id: &str) {
        self.edit("scroll_page_up", view_id, None)
    }

    pub fn page_up(&self, view_id: &str) {
        self.edit("page_up", view_id, None)
    }

    pub fn page_up_and_modify_selection(&self, view_id: &str) {
        self.edit("page_up_and_modify_selection", view_id, None)
    }

    pub fn scroll_page_down(&self, view_id: &str) {
        self.edit("scroll_page_down", view_id, None)
    }

    pub fn page_down(&self, view_id: &str) {
        self.edit("page_down", view_id, None)
    }

    pub fn page_down_and_modify_selection(&self, view_id: &str) {
        self.edit("page_down_and_modify_selection", view_id, None)
    }
}
