use serde;
use serde_json as json;

use errors::*;
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

fn deserialize_operation_type<'de, D>(de: D) -> ::std::result::Result<OperationType, D::Error>
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
    pub fn apply(&self, old_lines: &[Line], old_ix: u64, new_lines: &mut Vec<Line>) -> Result<u64> {
        // FIXME: this method panics if we don't check old_lines indices access.
        // we should check old_lines length and return an error when trying to access an out of
        // bound index.
        match self.operation_type {
            OperationType::Copy_ => {
                let new_ix = old_ix + self.nb_lines;
                debug!("copying line {} to {}", old_ix, new_ix);

                for i in old_ix..new_ix {
                    new_lines.push(old_lines[i as usize].clone());
                }
                Ok(new_ix)
            }
            OperationType::Skip => {
                debug!("skipping {} lines", self.nb_lines);
                Ok(old_ix + self.nb_lines)
            }
            OperationType::Invalidate => {
                let new_ix = old_ix + self.nb_lines;
                debug!("invalidating lines {} to {}", old_ix, new_ix);

                for i in 0..self.nb_lines {
                    let mut line = old_lines[i as usize].clone();
                    line.is_valid = false;
                    new_lines.push(line);
                }
                Ok(new_ix)
            }
            OperationType::Update => {
                let new_ix = old_ix + self.nb_lines;
                debug!("updating lines {} to {}", old_ix, new_ix);
                let lines = self.lines.clone().unwrap();
                for i in old_ix..new_ix {
                    let mut line = old_lines[i as usize].clone();
                    line.cursor = lines[i as usize].cursor.clone();
                    line.styles = lines[i as usize].styles.clone();
                    new_lines.push(line);
                }
                Ok(new_ix)
            }
            OperationType::Insert => {
                let lines = self.lines.clone().unwrap();
                new_lines.extend(lines.iter().cloned());
                Ok(old_ix)
            }
        }
    }
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
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
