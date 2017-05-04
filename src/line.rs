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
    /// Insert the cursors and handle styles (not implemented yet)
    pub fn render<W: Write>(&self, w: &mut W, lineno: u16, cursor: Option<&Cursor>) -> Result<()> {
        let mut line = self.text
                           .as_ref()
                           .map(|s| s.clone())
                           .unwrap_or(String::new());

        // Draw "fake" cursors. we have only one "real" cursor in a terminal so we render the
        // others manually by inserting escape sequences in the string.
        if let Some(ref cursors) = self.cursors {
            // We need to keep track of the escape sequences we add, since they count as 1
            // character and change the indices. One solution to avoid that would be to insert them
            // right to left.
            let mut offset: usize = 0;

            for idx in cursors {
                // If this cursor is the real cursor we don't want to draw it.
                // We skip it, and the cursor will be set here later.
                if let Some(real_cursor) = cursor {
                    if real_cursor.column as u64 == *idx {
                        continue;
                    }
                }

                // Make sure the cursor is within the bounds of the string. It *should* be the case
                // after padding, assuming the core sends correct updates, but better be safe than
                // sorry. Note that padding is necessary for cases where the cursor is at the end
                // of the line.
                self.pad(&mut line);
                let idx = offset + *idx as usize;
                if idx + 1 > line.len() {
                    error!(
                        "Cannot set cursor: cursor index {} is bigger than line length ({})",
                        idx,
                        line.len());
                    bail!(ErrorKind::DisplayError);
                }

                // Insert the "cursor".
                line.insert_str(idx + 1, &format!("{}", style::Reset));
                line.insert_str(idx, &format!("{}", style::Invert));
                offset += 2;
            }
        }
        write!(w, "{}{}{}", cursor::Goto(1, lineno), clear::CurrentLine, line)
            .chain_err(|| ErrorKind::DisplayError)?;
        w.flush().chain_err(|| ErrorKind::DisplayError)?;
        Ok(())
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
