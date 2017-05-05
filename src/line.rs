use std::io::Write;
use std::default::Default;

use termion::clear;
use termion::cursor;
use termion::style;

use cursor::Cursor;
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

    pub fn render<W: Write>(&self, w: &mut W, lineno: u16, cursor: Option<&Cursor>) -> Result<()> {
        let mut line = self.text.as_ref().cloned().unwrap_or_default();

        self.trim_new_line(&mut line);
        self.insert_cursors(&mut line, cursor);
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

    fn insert_cursors(&self, text: &mut String, real_cursor: Option<&Cursor>) {
        if let Some(ref cursors) = self.cursors {
            for idx in cursors.iter().rev() {
                let idx = *idx;
                // If this cursor is the real cursor we don't want to draw it.
                // We skip it, and the cursor will be set here later.
                if let Some(real_cursor) = real_cursor {
                    if real_cursor.column == idx {
                        continue;
                    }
                }

                // Make sure the cursor is within the bounds of the string by adding some padding.
                // Note that if this happens, it can only happen for the right-most cursor.
                if idx as usize + 1 > text.len() {
                    for _ in 0..idx as usize + 1 - text.len() {
                        text.push(' ');
                    }
                }

                // Insert the "cursor".
                text.insert_str(idx as usize + 1, &format!("{}", style::Reset));
                text.insert_str(idx as usize, &format!("{}", style::Invert));
            }
        }
    }
}
