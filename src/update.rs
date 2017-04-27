use serde_json;

use line::Line;

pub struct Update {
    pub lines: Vec<Line>,
    height: u64,
    pub scroll_to: (u64, u64),
    pub first_line: u64
}

impl Update {
    pub fn from_value(value: &serde_json::Value) -> Update {
        let object = value.as_object().unwrap();
        let scroll_to = object.get("scrollto").unwrap().as_array().unwrap();
        let mut lines: Vec<Line> = vec![];
        for line in object.get("lines").unwrap().as_array().unwrap().iter() {
            lines.push(Line::from_value(line));
        }
        Update {
            height: object.get("height").unwrap().as_u64().unwrap(),
            first_line: object.get("first_line").unwrap().as_u64().unwrap(),
            lines: lines,
            scroll_to: (scroll_to[0].as_u64().unwrap(), scroll_to[1].as_u64().unwrap()),
        }
    }
}
