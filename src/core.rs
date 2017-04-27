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

pub struct Core {
    stdin: ChildStdin,
    pub update_rx: mpsc::Receiver<Value>,
    rpc_rx: mpsc::Receiver<(u64,Value)>, // ! A simple piping works only for synchronous calls.
    rpc_index: u64,
    current_view: String,
    views: HashMap<String, View>,
}

impl Core {
    pub fn new(executable: &str, file: &str) -> Core {
        // spawn the core process
        let process = Command::new(executable)
                                .arg(file)
                                .stdout(Stdio::piped())
                                .stdin(Stdio::piped())
                                .stderr(Stdio::piped())
                                .env("RUST_BACKTRACE", "1")
                                .spawn()
                                .unwrap_or_else(|e| { panic!("failed to execute core: {}", e) });


        let (update_tx, update_rx) = mpsc::channel();
        let (rpc_tx, rpc_rx) = mpsc::channel();
        let stdout = process.stdout.unwrap();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if let Ok(data) = serde_json::from_slice::<Value>(line.unwrap().as_bytes()) {
                    let req = data.as_object().unwrap();
                    info!("<<< {:?}", req);
                    if let (Some(id), Some(result)) = (req.get("id"), req.get("result")) {
                        rpc_tx.send((id.as_u64().unwrap(), result.clone())).unwrap();
                        info!(">>> {:?}", (id.as_u64().unwrap(), result.clone()));
                    } else if let (Some(method), Some(params)) = (req.get("method"), req.get("params")) {
                        let meth = method.as_str().unwrap();
                        if meth == "set_style" || meth == "scroll_to" || meth == "update" {
                            let request = json!([method.clone(), params.clone()]);
                            update_tx.send(request).unwrap();
                        } else {
                            panic!("Unknown method {:?}.", method.as_str().unwrap());
                        }
                    } else {
                        panic!("Could not parse the core output: {:?}", req);
                    }
                }
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

        let mut core = Core {
            stdin: stdin,
            update_rx: update_rx,
            rpc_rx: rpc_rx,
            rpc_index: 0,
            current_view: "".into(),
            views: HashMap::new(),
        };
        let view_id = core.new_view(Some(file.to_string())).as_str().unwrap().to_string();
        let view = View::new(file.to_string());
        core.views.insert(view_id.clone(), view);
        core.current_view = view_id;
        core
    }

    pub fn update(&mut self, update: &Update) {
        let mut view = self.view();
        view.update(update);
        self.views.insert(self.current_view.clone(), view);
    }

    pub fn view(&self) -> View {
        self.views.clone().get(&self.current_view).unwrap().clone()
    }

    /// Build and send a JSON RPC request, returning the associated request ID to pair it with
    /// the response
    fn request(&mut self, method: &str, params: Value) -> u64 {
        self.rpc_index += 1;
        let message = json!({
            "id": self.rpc_index,
            "method": method,
            "params": params,
        });
        self.send(&message);
        self.rpc_index
    }

    /// Build and send a JSON RPC notification. No synchronous response is expected, so
    /// there is no ID.
    fn notify(&mut self, method: &str, params: Value) {
        let message = json!({
            "method": method,
            "params": params,
        });
        self.send(&message);
    }

    /// Serialize JSON object and send it to the server
    fn send(&mut self, message: &Value) {
        let mut str_msg = serde_json::ser::to_string(&message).unwrap();
        str_msg.push('\n');
        self.stdin.write(str_msg.as_bytes()).unwrap();
    }

    fn call_sync(&mut self, method: &str, params: Value) -> Value {
        let i = self.request(method, params);
        let (id,result) = self.rpc_rx.recv().unwrap();
        assert_eq!(i, id);
        result
    }

    fn call_edit(&mut self, method: &str, params: Option<Value>) {
        let msg = json!({
            "method": method,
            "view_id": &self.current_view,
            "params": params.unwrap_or(json!([])),
        });
        info!(">>> {:?}", msg);
        self.notify("edit", msg);
    }

    fn call_edit_sync(&mut self, method: &str, params: Option<Value>) -> Value{
        let msg = json!({
            "method": method,
            "view_id": &self.current_view,
            "params": params.unwrap_or(json!([])),
        });
        self.call_sync("edit", msg)
    }

    pub fn new_view(&mut self, file_path: Option<String>) -> Value {
        let mut msg: Value;
        if let Some(file_path) = file_path {
            msg = json!({"file_path": file_path});
        } else {
            msg = json!({});
        }
        self.call_sync("new_view", msg)
    }

    pub fn save(&mut self) {
        let views = self.views.clone();
        let save_params = json!({
            "view_id": &self.current_view.clone(),
            "file_path": views.get(&self.current_view).unwrap().filepath.clone(),
        });
        self.request("save", save_params);
    }

    pub fn open(&mut self, filename: &str) {
        self.call_edit("open", Some(json!({"filename": filename})));
    }

    pub fn left(&mut self) { self.call_edit("move_left", None); }
    pub fn left_sel(&mut self) { self.call_edit("move_left_and_modify_selection", None); }

    pub fn right(&mut self) { self.call_edit("move_right", None); }
    pub fn right_sel(&mut self) { self.call_edit("move_right_and_modify_selection", None); }

    pub fn up(&mut self) { self.call_edit("move_up", None); }
    pub fn up_sel(&mut self) { self.call_edit("move_up_and_modify_selection", None); }

    pub fn down(&mut self) { self.call_edit("move_down", None); }
    pub fn down_sel(&mut self) { self.call_edit("move_down_and_modify_selection", None); }

    pub fn del(&mut self) { self.call_edit("delete_backward", None); }

    pub fn page_up(&mut self) { self.call_edit("page_up", None); }
    pub fn page_up_sel(&mut self) { self.call_edit("page_up_and_modify_selection", None); }

    pub fn page_down(&mut self) { self.call_edit("page_down", None); }
    pub fn page_down_sel(&mut self) { self.call_edit("page_down_and_modify_selection", None); }

    pub fn insert_newline(&mut self) { self.call_edit("insert_newline", None); }

    pub fn f1(&mut self) { self.call_edit("debug_rewrap", None); }

    pub fn f2(&mut self) { self.call_edit("debug_test_fg_spans", None); }

    pub fn char(&mut self, ch: char) {
        self.call_edit("insert", Some(json!({"chars": ch})));
    }

    pub fn scroll(&mut self, start: u64, end: u64) {
        self.call_edit("scroll", Some(json!([start, end])));
    }

    pub fn click(&mut self, line: u64, column: u64) {
        self.call_edit("click", Some(json!([line, column, 0, 1])));
    }
    pub fn drag(&mut self, line: u64, column: u64) {
        self.call_edit("drag", Some(json!([line, column, 0, 1])));
    }

    pub fn copy(&mut self) -> String {
        self.call_edit_sync("copy", None).as_str().map(|x|x.into()).unwrap()
    }
    pub fn cut(&mut self) -> String {
        self.call_edit_sync("cut", None).as_str().map(|x|x.into()).unwrap()
    }
    pub fn paste(&mut self, s: String) {
        self.call_edit("insert", Some(json!({"chars": s})));
    }

    #[allow(dead_code)]
    pub fn test(&mut self) {
        self.render_lines(0, 10);
    }

    pub fn render_lines(&mut self, _start: u64, _end: u64) {
        unimplemented!()
        // self.rpc_index += 1;
        // println!("render_lines");
        // let value = ArrayBuilder::new()
        //     .push("rpc")
        //     .push_object(|builder| builder
        //         .insert("index", self.rpc_index)
        //         .insert_array("request", |builder| builder
        //             .push("render_lines")
        //             .push_object(|builder| builder
        //                 .insert("first_line", _start)
        //                 .insert("last_line", _end)
        //             )
        //         )
        //     ).unwrap();
        // self.write(value);
    }

    pub fn render_lines_sync(&mut self, _start: u64, _end: u64) -> Value {
        unimplemented!()
        // self.render_lines(_start, _end);
        // let value = self.rpc_rx.recv().unwrap();
        // let object = value.as_object().unwrap();
        // assert_eq!(self.rpc_index, object.get("index").unwrap().as_u64().unwrap());
        // object.get("result").unwrap().clone()
    }
}
