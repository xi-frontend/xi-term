use serde_json;

pub struct Line {
    pub text: String,
    pub selection: Option<(u64, u64)>,
    cursor: Option<u64>,
}

impl Line {
    pub fn from_value(value: &serde_json::Value) -> Line {
        let line_arr = value.as_array().unwrap();
        let mut line = Line {
            text: line_arr[0].as_str().unwrap().to_string(),
            cursor: None,
            selection: None,
        };
        for annotation in line_arr.iter().skip(1).map(|a| a.as_array().unwrap()) {
            match annotation[0].as_str().unwrap() {
                "cursor" => {
                    line.cursor = Some(annotation[1].as_u64().unwrap());
                },
                "sel" => {
                    line.selection = Some((annotation[1].as_u64().unwrap(), annotation[2].as_u64().unwrap()));
                },
                _ => {
                    error!("unknown annotation");
                }
            }
        }
        line
    }
}
