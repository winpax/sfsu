//! Implement gitoxide traits for indicatif progress bars

use std::sync::atomic::Ordering;

use gix::{progress::MessageLevel, Count, NestedProgress, Progress};

impl gix::progress::Progress for super::ProgressBar {
    fn init(
        &mut self,
        max: Option<gix::progress::prodash::progress::Step>,
        unit: Option<gix::progress::Unit>,
    ) {
        let max = max.unwrap_or_default() as u64;
        self.bar.set_length(max);

        if let Some(unit) = unit {
            self.set_unit(unit);
        }
    }

    fn set_name(&mut self, name: String) {
        self.bar.set_prefix(name);
    }

    fn name(&self) -> Option<String> {
        Some(self.bar.prefix())
    }

    fn id(&self) -> gix::progress::Id {
        [0, 0, 0, 0]
    }

    fn message(&self, level: MessageLevel, message: String) {
        match level {
            MessageLevel::Info => debug!("Received message: {message}"),
            MessageLevel::Failure | MessageLevel::Success => self.bar.finish_with_message(message),
        }
    }
}

impl Count for super::ProgressBar {
    fn set(&self, step: gix::progress::prodash::progress::Step) {
        self.bar.set_position(step as u64);
    }

    fn step(&self) -> gix::progress::prodash::progress::Step {
        self.step.load(Ordering::Relaxed)
    }

    fn inc_by(&self, step: gix::progress::prodash::progress::Step) {
        self.set_step(step);
    }

    fn counter(&self) -> gix::progress::StepShared {
        self.step.clone()
    }
}

impl NestedProgress for super::MultiProgressHandler {
    type SubProgress = super::MultiProgressHandler;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        self.add_child_with_id(name, [0, 0, 0, 0])
    }

    fn add_child_with_id(
        &mut self,
        name: impl Into<String>,
        id: gix::progress::Id,
    ) -> Self::SubProgress {
        let mut multi_progress = Self::new();

        let mut bar = super::ProgressBar::new(0);
        bar.id = id;
        bar.set_name(name.into());

        self.add(bar.clone());

        multi_progress.add(bar);

        multi_progress
    }
}

impl Progress for super::MultiProgressHandler {
    fn init(
        &mut self,
        max: Option<gix::progress::prodash::progress::Step>,
        unit: Option<gix::progress::Unit>,
    ) {
        for bar in &mut self.bars {
            bar.init(max, unit.clone());
        }
    }

    fn set_name(&mut self, name: String) {
        for bar in &mut self.bars {
            bar.set_name(name.clone());
        }
    }

    fn name(&self) -> Option<String> {
        for bar in &self.bars {
            if let Some(name) = bar.name() {
                return Some(name);
            }
        }

        None
    }

    fn id(&self) -> gix::progress::Id {
        self.bars[0].id
    }

    fn message(&self, level: MessageLevel, message: String) {
        for bar in &self.bars {
            bar.message(level, message.clone());
        }
    }
}

impl Count for super::MultiProgressHandler {
    fn set(&self, step: gix::progress::prodash::progress::Step) {
        for bar in &self.bars {
            bar.set(step);
        }
    }

    fn step(&self) -> gix::progress::prodash::progress::Step {
        self.bars[0].step()
    }

    fn inc_by(&self, step: gix::progress::prodash::progress::Step) {
        for bar in &self.bars {
            bar.inc_by(step);
        }
    }

    fn counter(&self) -> gix::progress::StepShared {
        self.bars[0].counter()
    }
}
