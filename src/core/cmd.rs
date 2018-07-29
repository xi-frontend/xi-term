use xrl::ViewId;

/// Command system for xi-term.
/// A command represents a task the user wants the editor to preform,
/// currently commands can only be input through the CommandPrompt. Vim style.
#[derive(Debug)]
pub enum Command {
    /// Close the CommandPrompt
    Cancel,
    /// Quit editor.
    Quit,
    /// Save the current file buffer
    Save(Option<ViewId>),
    Invalid(String),
}
