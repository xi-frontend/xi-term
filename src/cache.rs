use errors::*;
use line::Line;
use update::Update;

#[derive(Clone, Debug)]
pub struct LineCache {
    lines: Vec<Line>,
    dirty: bool,
}

impl LineCache {
    pub fn new() -> Self {
        LineCache {
            lines: vec![],
            dirty: true,
        }
    }
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn lines(&self) -> &[Line] {
        self.lines.as_slice()
    }

    pub fn update(&mut self, update: &Update) -> Result<()> {
        let mut lines = vec![];
        let mut index = 0;

        for operation in &update.operations {
            index = operation.apply(&self.lines, index, &mut lines)?;
        }

        self.lines = lines;
        self.dirty = true;
        Ok(())
    }
}
