use std::io::Write;
use std::default::Default;

use termion;
use termion::clear;
use termion::cursor;

use errors::*;

fn _return_true() -> bool {
    true
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Line {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    #[serde(rename="cursor")]
    pub cursors: Vec<u64>,
    #[serde(default)]
    pub styles: Vec<i64>,
    #[serde(default="_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            text: "".to_owned(),
            cursors: vec![],
            styles: vec![],
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
        let mut line = self.text.clone();
        self.trim_new_line(&mut line);
        self.add_styles(&mut line)?;
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

    fn add_styles(&self, text: &mut String) -> Result<()> {
        if self.styles.is_empty() {
            return Ok(());
        }
        if self.styles.len() % 3 != 0 {
            error!("Invalid style array (should be a multiple of 3)");
            bail!(ErrorKind::DisplayError);
        }
        let mut style_idx = 0;
        loop {
            let start = self.styles[style_idx] as usize;
            let end = start + self.styles[style_idx + 1] as usize;

            if end >= text.len() {
                text.push_str(&format!("{}", termion::style::Reset));
            } else {
                text.insert_str(end, &format!("{}", termion::style::Reset));
            }
            text.insert_str(start, &format!("{}", termion::style::Invert));

            style_idx += 3;
            if style_idx >= self.styles.len() {
                break;
            }
        }
        Ok(())
    }
}
