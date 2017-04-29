use std::default::Default;

fn _return_true() -> bool {
    true
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    pub text: Option<String>,
    pub cursor: Option<Vec<u64>>,
    pub styles: Option<Vec<i64>>,
    #[serde(default="_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            text: None,
            cursor: None,
            styles: None,
            is_valid: true,
        }
    }
}

impl Line {
    pub fn invalid() -> Line {
        Line {
            is_valid: false,
            ..Default::default()
        }
    }
}
