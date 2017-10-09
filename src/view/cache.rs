use xrl::{Line, Operation, OperationType, Update};

#[derive(Clone, Debug)]
pub struct LineCache {
    pub invalid_before: u64,
    pub lines: Vec<Line>,
    pub invalid_after: u64,
}

impl LineCache {
    pub fn new() -> Self {
        LineCache {
            invalid_before: 0,
            lines: vec![],
            invalid_after: 0,
        }
    }

    pub fn update(&mut self, update: Update) {
        let LineCache {
            ref mut lines,
            ref mut invalid_before,
            ref mut invalid_after,
        } = *self;
        let helper = UpdateHelper {
            old_lines: lines,
            old_invalid_before: invalid_before,
            old_invalid_after: invalid_after,
            new_lines: Vec::new(),
            new_invalid_before: 0,
            new_invalid_after: 0,
        };
        helper.update(update.operations);
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

struct UpdateHelper<'a, 'b, 'c> {
    old_lines: &'a mut Vec<Line>,
    old_invalid_before: &'b mut u64,
    old_invalid_after: &'c mut u64,
    new_lines: Vec<Line>,
    new_invalid_before: u64,
    new_invalid_after: u64,
}

impl<'a, 'b, 'c> UpdateHelper<'a, 'b, 'c> {
    fn apply_copy(&mut self, nb_lines: u64) {
        info!("copying {} lines", nb_lines);
        let UpdateHelper {
            ref mut old_lines,
            ref mut old_invalid_before,
            ref mut old_invalid_after,
            ref mut new_lines,
            ref mut new_invalid_before,
            ref mut new_invalid_after,
            ..
        } = *self;
        let mut nb_lines = nb_lines;

        // Copy invalid lines that precede the valid ones
        if **old_invalid_before > nb_lines {
            **old_invalid_before -= nb_lines;
            *new_invalid_before += nb_lines;
            return;
        } else if **old_invalid_after > 0 {
            nb_lines -= **old_invalid_before;
            *new_invalid_before += **old_invalid_before;
            **old_invalid_before = 0;
        }

        // Copy the valid lines
        let nb_valid_lines = old_lines.len();
        if nb_lines < nb_valid_lines as u64 {
            new_lines.extend(old_lines.drain(0..nb_lines as usize));
            return;
        } else {
            new_lines.extend(old_lines.drain(..));
            nb_lines -= nb_valid_lines as u64;
        }

        // Copy the remaining invalid lines
        if **old_invalid_after >= nb_lines {
            **old_invalid_after -= nb_lines;
            *new_invalid_after += nb_lines;
            return;
        }

        error!(
            "{} lines left to copy, but only {} lines in the old cache",
            nb_lines,
            **old_invalid_after
        );
        panic!("cache update failed");
    }

    fn apply_skip(&mut self, nb_lines: u64) {
        info!("skipping {} lines", nb_lines);

        let UpdateHelper {
            ref mut old_lines,
            ref mut old_invalid_before,
            ref mut old_invalid_after,
            ..
        } = *self;

        let mut nb_lines = nb_lines;

        // Skip invalid lines that comes before the valid ones.
        if **old_invalid_before > nb_lines {
            **old_invalid_before -= nb_lines;
            return;
        } else if **old_invalid_before > 0 {
            nb_lines -= **old_invalid_before;
            **old_invalid_before = 0;
        }

        // Skip the valid lines
        let nb_valid_lines = old_lines.len();
        if nb_lines < nb_valid_lines as u64 {
            old_lines.drain(0..nb_lines as usize).last();
            return;
        } else {
            old_lines.drain(..).last();
            nb_lines -= nb_valid_lines as u64;
        }

        // Skip the remaining invalid lines
        if **old_invalid_after >= nb_lines {
            **old_invalid_after -= nb_lines;
            return;
        }

        error!(
            "{} lines left to skip, but only {} lines in the old cache",
            nb_lines,
            **old_invalid_after
        );
        panic!("cache update failed");
    }

    fn apply_invalidate(&mut self, nb_lines: u64) {
        info!("invalidating {} lines", nb_lines);
        if self.new_lines.is_empty() {
            self.new_invalid_before += nb_lines;
        } else {
            self.new_invalid_after += nb_lines;
        }
    }

    fn apply_insert(&mut self, mut lines: Vec<Line>) {
        info!("inserting {} lines", lines.len());
        self.new_lines.extend(lines.drain(..).map(|mut line| {
            trim_new_line(&mut line.text);
            line
        }));
    }

    fn apply_update(&mut self, nb_lines: u64, lines: Vec<Line>) {
        info!("updating {} lines", nb_lines);
        let UpdateHelper {
            ref mut old_lines,
            ref mut new_lines,
            ..
        } = *self;
        if nb_lines > old_lines.len() as u64 {
            error!(
                "{} lines to update, but only {} lines in cache",
                nb_lines,
                old_lines.len()
            );
            panic!("failed to update the cache");
        }
        new_lines.extend(
            old_lines
                .drain(0..nb_lines as usize)
                .zip(lines.into_iter())
                .map(|(mut old_line, update)| {
                    old_line.cursor = update.cursor;
                    old_line.styles = update.styles;
                    old_line
                }),
        )
    }

    fn update(mut self, operations: Vec<Operation>) {
        for op in operations {
            match op.operation_type {
                OperationType::Copy_ => (&mut self).apply_copy(op.nb_lines),
                OperationType::Skip => (&mut self).apply_skip(op.nb_lines),
                OperationType::Invalidate => (&mut self).apply_invalidate(op.nb_lines),
                OperationType::Insert => (&mut self).apply_insert(op.lines),
                OperationType::Update => (&mut self).apply_update(op.nb_lines, op.lines),
            }
        }
        *self.old_lines = self.new_lines;
        *self.old_invalid_before = self.new_invalid_before;
        *self.old_invalid_after = self.new_invalid_after;
    }
}

fn trim_new_line(text: &mut String) {
    if let Some('\n') = text.chars().last() {
        text.pop();
    }
}
