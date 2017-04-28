use serde;
use serde_json as json;
use serde_derive;
use line::Line;

#[derive(Deserialize, Debug, PartialEq)]
pub enum OperationType {
    Copy_,
    Skip,
    Invalidate,
    Update,
    Insert,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Operation {
    #[serde(rename="op")]
    #[serde(deserialize_with="deserialize_operation_type")]
    pub operation_type: OperationType,
    #[serde(rename="n")]
    pub nb_lines: u64,
    pub lines: Option<Vec<Line>>,
}

fn deserialize_operation_type<'de, D>(de: D) -> Result<OperationType, D::Error>
    where D: serde::Deserializer<'de>
{
    let value: json::Value = try!(serde::Deserialize::deserialize(de));
    match value {
        json::Value::String(ref s) if &*s == "copy" => Ok(OperationType::Copy_),
        json::Value::String(ref s) if &*s == "skip" => Ok(OperationType::Skip),
        json::Value::String(ref s) if &*s == "invalidate" => Ok(OperationType::Invalidate),
        json::Value::String(ref s) if &*s == "update" => Ok(OperationType::Update),
        json::Value::String(ref s) if &*s == "ins" => Ok(OperationType::Insert),
        _ => Err(serde::de::Error::custom("Unexpected value for operation type")),
    }
}

impl Operation {
    pub fn apply(&self, old_lines: &[Line], old_line_index: u64, new_lines: &mut Vec<Line>) -> u64 {
        match self.operation_type {
            OperationType::Copy_ => {
                let new_index = old_line_index + self.nb_lines;
                for i in old_line_index..new_index {
                    new_lines.push(old_lines[i as usize].clone());
                }
                new_index
            }
            OperationType::Skip => old_line_index + self.nb_lines,
            OperationType::Invalidate => {
                let new_index = old_line_index + self.nb_lines;
                for _ in 0..self.nb_lines {
                    new_lines.push(Line::invalid());
                }
                new_index
            }
            OperationType::Update => {
                let new_index = old_line_index + self.nb_lines;
                let lines = self.lines.clone().unwrap();
                for i in old_line_index..new_index {
                    let mut line = old_lines[i as usize].clone();
                    line.cursor = lines[i as usize].cursor.clone();
                    line.styles = lines[i as usize].styles.clone();
                    new_lines.push(line);
                }
                new_index
            }
            OperationType::Insert => {
                let lines = self.lines.clone().unwrap();
                new_lines.extend(lines.iter().cloned());
                old_line_index + self.nb_lines
            }
        }
    }
}

#[test]
fn deserialize_operation_from_value() {
    use serde_json;

    let value = json!({"n": 12, "op": "ins"});
    let operation = Operation {
        operation_type: OperationType::Insert,
        nb_lines: 12,
        lines: None,
    };
    let deserialized: Result<Operation, _> = serde_json::from_value(value);
    assert_eq!(deserialized.unwrap(), operation);

    let value = json!({"lines":[{"cursor":[0],"styles":[],"text":"foo"},{"styles":[],"text":""}],"n":60,"op":"invalidate"});
    let operation = Operation {
        operation_type: OperationType::Invalidate,
        nb_lines: 60,
        lines: Some(vec![
            Line { cursor: Some(vec![0]), styles: Some(vec![]), text: Some("foo".to_owned()) },
            Line { cursor: None, styles: Some(vec![]), text: Some("".to_owned()) }]),
    };
    let deserialized: Result<Operation, _> = serde_json::from_value(value);
    assert_eq!(deserialized.unwrap(), operation);
}

#[test]
fn deserialize_operation() {
    use serde_json;

    let s = r#"{"n": 12, "op": "ins"}"#;
    let operation = Operation {
        operation_type: OperationType::Insert,
        nb_lines: 12,
        lines: None,
    };
    let deserialized: Result<Operation, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), operation);


    let s = r#"{"lines":[{"cursor":[0],"styles":[],"text":"foo"},{"styles":[],"text":""}],"n":60,"op":"invalidate"}"#;
    let operation = Operation {
        operation_type: OperationType::Invalidate,
        nb_lines: 60,
        lines: Some(vec![
            Line { cursor: Some(vec![0]), styles: Some(vec![]), text: Some("foo".to_owned()) },
            Line { cursor: None, styles: Some(vec![]), text: Some("".to_owned()) }]),
    };
    let deserialized: Result<Operation, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), operation);
}

