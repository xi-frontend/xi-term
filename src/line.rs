use std::io::Write;
use std::default::Default;

use termion::clear;
use termion::cursor;

use errors::*;

fn _return_true() -> bool {
    true
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    pub text: Option<String>,
    #[serde(rename="cursor")]
    pub cursors: Option<Vec<u64>>,
    pub styles: Option<Vec<i64>>,
    #[serde(default="_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            text: None,
            cursors: None,
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

    pub fn render<W: Write>(&self, w: &mut W, lineno: u16) -> Result<()> {
        let mut line = self.text.as_ref().cloned().unwrap_or_default();

        self.trim_new_line(&mut line);
        write!(w, "{}{}{}", cursor::Goto(1, lineno), clear::CurrentLine, line)
            .chain_err(|| ErrorKind::DisplayError)?;
        w.flush().chain_err(|| ErrorKind::DisplayError)?;
        Ok(())
    }

    fn trim_new_line(&self, text: &mut String) {
        if let Some('\n') = text.chars().last() {
            text.pop();
        }
    }
}
