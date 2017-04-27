use serde_json;
use serde_json::value::to_value;

use op::Op;

pub struct Update {
    pub rev: u64,
    pub ops: Vec<Op>,
}

impl Update {
    pub fn from_value(value: &serde_json::Value) -> Update {
        let object = value.as_object().unwrap();
        let ops = object.get("ops")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|op| Op::from_value(op))
            .collect();
        let rev = object.get("rev").unwrap_or(&to_value(0).unwrap()).as_u64().unwrap();
        Update {
            rev: rev,
            ops: ops,
        }
    }
}
