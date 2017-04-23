use serde_json;

pub struct Line {
    text: Option<String>,
    cursor: Option<Vec<u64>>,
    styles: Option<Vec<i64>>,
}

impl Line {
    pub fn from_value(value: &serde_json::Value) -> Line {
        let obj = value.as_object().unwrap();
        let text = match obj.get("text") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            _ => None,
        };
        let cursor = match obj.get("cursor") {
            Some(cursor) => {
                Some(cursor.as_array().unwrap().iter().map(|c| c.as_u64().unwrap()).collect())
            },
            _ => None,
        };
        let styles = match obj.get("styles") {
            Some(styles) => {
                Some(styles.as_array().unwrap().iter().map(|s| s.as_i64().unwrap()).collect())
            },
            _ => None,
        };
        Line {
            text: text,
            cursor: cursor,
            styles: styles,
        }
    }
}
