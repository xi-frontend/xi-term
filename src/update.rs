use serde_json;

use op::Op;

pub struct Update {
    rev: u64,
    ops: Vec<Op>,
}

impl Update {
    pub fn from_value(value: &serde_json::Value) -> Update {
        let object = value.as_object().unwrap();
        let ops = object.get("ops").unwrap().as_array().unwrap().iter().map(
            |op| Op::from_value(op)
        ).collect();
        Update {
            rev: object.get("rev").unwrap().as_u64().unwrap(),
            ops: ops,
        }
    }
}
