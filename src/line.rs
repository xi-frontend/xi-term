use std::default::Default;

use termion::style;

use errors::*;

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

    /// Insert the cursors and handle styles (not implemented yet)
    pub fn render(&self) -> Result<String> {
        let mut line = self.text
                           .as_ref()
                           .map(|s| s.clone())
                           .unwrap_or(String::new());
        self.pad(&mut line);
        if let Some(cursors) = self.cursor.as_ref() {
            let mut offset: usize = 0;
            for idx in cursors {
                let idx = offset + *idx as usize;
                if idx + 1 > line.len() {
                    error!("Cannot set cursor: cursor index {} is bigger than line length ({})", idx, line.len());
                    bail!(ErrorKind::DisplayError);
                }
                line.insert_str(idx + 1, &format!("{}", style::Reset));
                line.insert_str(idx, &format!("{}", style::Invert));
                offset += 2;
            }
        }
        Ok(line)
    }

    fn pad(&self, text: &mut String) {
        if let Some('\n') = text.chars().last() {
            text.pop();
            text.push_str(" \n");
        } else {
            text.push(' ');
        }
    }
}
