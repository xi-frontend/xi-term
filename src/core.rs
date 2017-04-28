use std::process::{Stdio,Command,ChildStdin,ChildStdout};
use std::collections::HashMap;
use std::thread;
use std::io::{self, BufReader};
use std::io::prelude::*;
use xi_rpc::{self, Handler, RpcLoop, RpcPeer, RpcCtx};
use screen::Screen;
use input::{self, Input};

#[macro_use]
use serde_json::{self, Value};
use view::View;


pub struct Core {
    peer: RpcPeer<ChildStdin>,
    views: HashMap<String, View>,
    input: Input,
    screen: Screen,
}

impl Handler<ChildStdin> for Core {
    fn handle_notification(&mut self, ctx: RpcCtx<ChildStdin>, method: &str, params: &Value) {
        info!("<<< {}: {:?}", method, params);
        unimplemented!()
    }

    fn handle_request(&mut self, ctx: RpcCtx<ChildStdin>, method: &str, params: &Value) -> Result<Value, Value> {
        match method {
            "update" => {
                unimplemented!()
            }
            _ => unimplemented!()
        }
        info!("<<< {}: {:?}", method, params);
        unimplemented!()
    }

    fn idle(&mut self, ctx: RpcCtx<ChildStdin>, _token: usize) {
        info!("idling");
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
        }
    }

    fn request(&self, method: &str, params: &Value) -> Result<Value, xi_rpc::Error> {
        self.peer.send_rpc_request(method, params)
    }

    fn notify(&self, method: &str, params: Value) {
        self.peer.send_rpc_notification(method, &params);
    }

    pub fn new_view(&self, path: Option<String>) -> Result<String, ()> {
        self.request("new_view", &json!({"file_path": path}))
            .map(|value| unimplemented!())
            .map_err(|_| ())
    }

    pub fn close_view(&self, view_id: &str) -> Result<(), ()> {
        self.request("close_view", &json!({"view_id": view_id}))
            .map(|_| ())
            .map_err(|_| ())
    }

    pub fn insert(&self, chars: &str, view_id: &str) -> Result<(), ()> {
        self.request("edit", &json!({
            "method": "insert",
            "view_id": view_id,
            "params": {"chars": chars},
        }))
            .map(|_| ())
            .map_err(|_| ())
    }
}
