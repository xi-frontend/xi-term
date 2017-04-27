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

    pub fn apply(&self, old_lines: &[Line], old_line_index: u64, new_lines: &mut Vec<Line>) -> u64 {
        match self.op {
            OpType::Cpy => {
                let new_index = old_line_index + self.n;
                for i in old_line_index..new_index {
                    new_lines.push(old_lines[i as usize].clone());
                }
                new_index
            },
            OpType::Skip => {
                old_line_index + self.n
            },
            OpType::Invalidate => {
                let new_index = old_line_index + self.n;
                for _ in 0..self.n {
                    new_lines.push(Line::invalid());
                }
                new_index
            },
            OpType::Update => {
                let new_index = old_line_index + self.n;
                let lines = self.lines.clone().unwrap();
                for i in old_line_index..new_index {
                    let mut line = old_lines[i as usize].clone();
                    line.cursor = lines[i as usize].cursor.clone();
                    line.styles = lines[i as usize].styles.clone();
                    new_lines.push(line);
                }
                new_index
            },
            OpType::Ins => {
                let lines = self.lines.clone().unwrap();
                new_lines.extend(lines.iter().cloned());
                old_line_index + self.n
            },
        }
    }
}
