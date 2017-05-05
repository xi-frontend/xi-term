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
    pub text: Option<String>,
    #[serde(rename="cursor")]
    pub cursors: Option<Vec<u64>>,
    pub styles: Vec<i64>,
    #[serde(default="_return_true")]
    #[serde(skip_deserializing)]
    pub is_valid: bool,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            text: None,
            cursors: None,
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
        let mut line = self.text.as_ref().cloned().unwrap_or_default();
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
            // FIXME: find a more descriptive error for this. Hopefully, this should not happen
            // though.
            error!("Invalid style");
            bail!(ErrorKind::DisplayError);
        }
        let mut style_idx = 0;
        loop {
            let start = self.styles[style_idx] as usize;
            let mut end = start + self.styles[style_idx + 1] as usize;

            if end > text.len() {
                // XXX: when dragging to a line below, xi-core sends a style that ends 1 index
                // after the end of the string. I think it considers the \n as a character, but we
                // remove it when rendering the line. Not sure if this is a bug. Should talk about
                // that on IRC.
                error!("style ends after the end of the line ({} > {})", end, text.len());
                end = text.len();
            }

            if end == text.len() {
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
