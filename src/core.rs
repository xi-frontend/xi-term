#![allow(dead_code)]
use std::collections::hash_map::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

use serde_json;
use serde_json::Value;

use update::Update;
use view::View;
use errors::*;

pub struct Core {
    stdin: ChildStdin,
    pub update_rx: mpsc::Receiver<Value>,
    rpc_rx: mpsc::Receiver<(u64, ::std::result::Result<Value, Value>)>,
    rpc_index: u64,
    current_view: String,
    views: HashMap<String, View>,
}

impl Core {
    pub fn new(executable: &str) -> Core {
        // spawn the core process
        let process = Command::new(executable)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .env("RUST_BACKTRACE", "1")
            .spawn()
            .unwrap_or_else(|e| panic!("failed to execute core: {}", e));


        let (update_tx, update_rx) = mpsc::channel();
        let (rpc_tx, rpc_rx) = mpsc::channel();

        let stdout = process.stdout.unwrap();

        thread::spawn(move || for line in BufReader::new(stdout).lines() {
            let line = line.unwrap();
            info!("<<< {}", line);

            if let Ok(data) = serde_json::from_slice::<Value>(line.as_bytes()) {
                let req = data.as_object().unwrap();

                if let (Some(id), Some(result)) = (req.get("id"), req.get("result")) {
                    rpc_tx
                        .send((id.as_u64().unwrap(), Ok(result.clone())))
                        .unwrap();
                    continue;
                }

                if let (Some(error), Some(id)) = (req.get("error"), req.get("id")) {
                    rpc_tx
                        .send((id.as_u64().unwrap(), Err(error.clone())))
                        .unwrap();
                    continue;
                }

                if let (Some(method), Some(params)) = (req.get("method"), req.get("params")) {
                    match method.as_str().unwrap() {
                        "set_style" | "scroll_to" | "update" => {
                            update_tx.send(json!([method, &params])).unwrap();
                        }
                        _ => {
                            error!("Unknown method {:?}.", method.as_str().unwrap());
                        }
                    }
                    continue;
                }

                error!("Unhandled core request");
            } else {
                error!("Could deserialize core output as a json object");
            }
        });

        let stderr = process.stderr.unwrap();

        thread::spawn(move || {
            let buf_reader = BufReader::new(stderr);
            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    error!("[core] {}", line);
                }
            }
        });

        let stdin = process.stdin.unwrap();

        Core {
            stdin: stdin,
            update_rx: update_rx,
            rpc_rx: rpc_rx,
            rpc_index: 0,
            current_view: "".into(),
            views: HashMap::new(),
        }
    }

    pub fn update(&mut self, update: &Update) -> Result<()> {
        // FIXME: I think view_id should be an argument here.
        // We can have updates for any view, not only the current one.
        info!("Updating current view");

        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.update_lines(update)
        } else {
            error!("View {} not found", &self.current_view);
            bail!(ErrorKind::UpdateError);
        }
    }

    pub fn scroll_to(&mut self, cursor: (u64, u64)) -> Result<()> {
        info!("Updating cursor position");

        if let Some(view) = self.get_view_mut() {
            view.update_cursor(cursor);
            return Ok(());
        }
        error!("View {} not found", self.current_view.as_str());
        bail!(ErrorKind::UpdateError);
    }

    pub fn get_view(&self) -> Option<&View> {
        self.views.get(&self.current_view)
    }

    pub fn get_view_mut(&mut self) -> Option<&mut View> {
        self.views.get_mut(&self.current_view)
    }

    /// Build and send a JSON RPC request, returning the associated request ID to pair it with
    /// the response
    fn request(&mut self, method: &str, params: Value) -> Result<u64> {
        self.rpc_index += 1;
        let message = json!({
            "id": self.rpc_index,
            "method": method,
            "params": params,
        });
        self.send(&message)?;
        Ok(self.rpc_index)
    }

    /// Build and send a JSON RPC notification. No synchronous response is expected, so
    /// there is no ID.
    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        let message = json!({
            "method": method,
            "params": params,
        });
        self.send(&message)
    }

    /// Serialize JSON object and send it to the server
    fn send(&mut self, message: &Value) -> Result<()> {
        let mut str_msg = serde_json::to_string(&message).chain_err(|| {
            error!("could not serialize the message to send");
            ErrorKind::RpcError
        })?;
        info!(">>> {}", &str_msg);
        str_msg.push('\n');
        self.stdin.write_all(str_msg.as_bytes()).chain_err(|| {
            error!("could not write the message to send");
            ErrorKind::RpcError
        })?;
        Ok(())
    }

    fn call_sync(&mut self, method: &str, params: Value) -> Result<Value> {
        let i = self.request(method, params)?;
        let (id, result) = self.rpc_rx.recv().unwrap();
        assert_eq!(i, id);
        // TODO: in the future, the caller should handle the error. For now we just log it and move
        // on, returning a generic RpcError.
        result.or_else(|_| {
            error!("synchronous call returned with an error");
            Err(ErrorKind::RpcError.into())
        })
    }

    fn call_edit(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let msg = json!({
            "method": method,
            "view_id": &self.current_view,
            "params": params.unwrap_or_else(|| Value::Array(vec![])),
        });
        self.notify("edit", msg)
    }

    fn call_edit_sync(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let msg = json!({
            "method": method,
            "view_id": &self.current_view,
            "params": params.unwrap_or_else(|| Value::Array(vec![])),
        });
        self.call_sync("edit", msg)
    }

    pub fn new_view(&mut self, file_path: Option<String>) -> Result<String> {
        let msg: Value;
        if let Some(file_path) = file_path {
            msg = json!({ "file_path": file_path });
        } else {
            msg = json!({});
        }
        self.call_sync("new_view", msg)?
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| {
                error!("Failed to deserialize \"new_view\" response as string");
                ErrorKind::RpcError.into()
            })
    }

    pub fn save(&mut self) -> Result<Value> {
        let views = self.views.clone();
        let save_params = json!({
            "view_id": &self.current_view.as_str(),
            "file_path": &views[&self.current_view].filepath.clone(),
        });
        self.call_sync("save", save_params)
    }

    pub fn left(&mut self) -> Result<()> {
        self.call_edit("move_left", None)
    }

    pub fn left_sel(&mut self) -> Result<()> {
        self.call_edit("move_left_and_modify_selection", None)
    }

    pub fn right(&mut self) -> Result<()> {
        self.call_edit("move_right", None)
    }

    pub fn right_sel(&mut self) -> Result<()> {
        self.call_edit("move_right_and_modify_selection", None)
    }

    pub fn up(&mut self) -> Result<()> {
        self.call_edit("move_up", None)
    }

    pub fn up_sel(&mut self) -> Result<()> {
        self.call_edit("move_up_and_modify_selection", None)
    }

    pub fn down(&mut self) -> Result<()> {
        self.call_edit("move_down", None)
    }

    pub fn down_sel(&mut self) -> Result<()> {
        self.call_edit("move_down_and_modify_selection", None)
    }

    pub fn del(&mut self) -> Result<()> {
        self.call_edit("delete_backward", None)
    }

    pub fn page_up(&mut self) -> Result<()> {
        self.call_edit("page_up", None)
    }

    pub fn page_up_sel(&mut self) -> Result<()> {
        self.call_edit("page_up_and_modify_selection", None)
    }

    pub fn page_down(&mut self) -> Result<()> {
        self.call_edit("page_down", None)
    }
    pub fn page_down_sel(&mut self) -> Result<()> {
        self.call_edit("page_down_and_modify_selection", None)
    }

    pub fn insert_newline(&mut self) -> Result<()> {
        self.call_edit("insert_newline", None)
    }

    pub fn f1(&mut self) -> Result<()> {
        self.call_edit("debug_rewrap", None)
    }

    pub fn f2(&mut self) -> Result<()> {
        self.call_edit("debug_test_fg_spans", None)
    }

    pub fn char(&mut self, ch: char) -> Result<()> {
        self.call_edit("insert", Some(json!({ "chars": ch })))
    }

    pub fn scroll(&mut self, start: u64, end: u64) -> Result<()> {
        self.call_edit("scroll", Some(json!([start, end])))
    }

    pub fn resize(&mut self, height: u16) -> Result<()> {
        let scroll_region: (u64, u64);
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.resize(height);
            scroll_region = view.get_window();
        } else {
            error!("View {} not found", &self.current_view);
            bail!(ErrorKind::UpdateError);
        }
        self.scroll(scroll_region.0, scroll_region.1)
    }

    pub fn click(&mut self, line: u64, column: u64) -> Result<()> {
        let lineno: u64;
        if let Some(view) = self.views.get_mut(&self.current_view) {
            lineno = line + view.get_window().0;
        } else {
            error!("View {} not found", &self.current_view);
            bail!(ErrorKind::UpdateError);
        }
        self.call_edit("click", Some(json!([lineno, column, 0, 1])))
    }

    pub fn drag(&mut self, line: u64, column: u64) -> Result<()> {
        let lineno: u64;
        if let Some(view) = self.views.get_mut(&self.current_view) {
            lineno = line + view.get_window().0;
        } else {
            error!("View {} not found", &self.current_view);
            bail!(ErrorKind::UpdateError);
        }
        self.call_edit("drag", Some(json!([lineno, column, 0, 1])))
    }

    pub fn copy(&mut self) -> Result<String> {
        self.call_edit_sync("copy", None)?
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| {
                error!("Failed to deserialize \"copy\" response as string");
                ErrorKind::RpcError.into()
            })
    }

    pub fn cut(&mut self) -> Result<String> {
        self.call_edit_sync("cut", None)?
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| {
                error!("Failed to deserialize \"cut\" response as string");
                ErrorKind::RpcError.into()
            })
    }

    pub fn paste(&mut self, s: String) -> Result<()> {
        self.call_edit("insert", Some(json!({ "chars": s })))
    }

    pub fn open(&mut self, filename: &str) -> Result<()> {
        let view_id = self.new_view(Some(filename.to_owned()))?;
        let view = View::new(filename);
        self.views.insert(view_id.clone(), view);
        self.current_view = view_id;
        Ok(())
    }
}
