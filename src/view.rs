use line::Line;
use update::Update;

#[derive(Clone)]
pub struct View {
    last_rev: u64,
    pub filepath: String,
    lines: Vec<Line>,
}

impl View {
    pub fn new(filepath: String) -> View {
        View {
            last_rev: 0,
            filepath: filepath,
            lines: vec![],
        }
    }

    pub fn update(&mut self, update: &Update) {
        if self.last_rev > update.rev {
            return;
        }
        let mut lines = vec![];
        let mut index = 0;

        for op in update.ops.iter() {
            index = op.apply(&self.lines, index, &mut lines);
        }

        self.last_rev = update.rev;
        self.lines = lines;
    }
}
