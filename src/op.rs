use serde_json;

use line::Line;

pub enum OpType {
    Cpy,
    Skip,
    Invalidate,
    Update,
    Ins,
}

impl OpType {
    fn from_str(op: &str) -> OpType {
        if op == "copy" {
            OpType::Cpy
        } else if op == "skip" {
            OpType::Skip
        } else if op == "invalidate" {
            OpType::Invalidate
        } else if op == "update" {
            OpType::Update
        } else if op == "ins" {
            OpType::Ins
        } else {
            panic!("Unknown Op type {:?}", op);
        }
    }
}

pub struct Op {
    op: OpType,
    n: u64,
    lines: Option<Vec<Line>>,
}

impl Op {
    pub fn from_value(value: &serde_json::Value) -> Op {
        let obj = value.as_object().unwrap();
        let lines = match obj.get("lines") {
            Some(line_value) => {
                let line_list = line_value.as_array().unwrap();
                Some(line_list.iter().map(|line| Line::from_value(line)).collect())
            },
            _ => None,
        };
        Op {
            op: OpType::from_str(obj.get("op").unwrap().as_str().unwrap()),
            n: obj.get("n").unwrap().as_u64().unwrap(),
            lines: lines,
        }
    }
}
