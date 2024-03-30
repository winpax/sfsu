use std::{fs::File, io::Write};

pub mod panics;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub fn init() -> Result<(), log::SetLoggerError> {
        let logger = Box::<Logger>::default();
        let logger = Box::leak(logger);

        log::set_logger(logger)?;

        trace!("Initialized logger");

        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(sfsu::Scoop::new_log().expect("new log"))
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        // metadata.level() <= log::Level::Info
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            writeln!(&self.file, "{}: {}", record.level(), record.args())
                .expect("writing to log file");
        }
    }

    fn flush(&self) {
        self.file
            .try_clone()
            .expect("cloning log file")
            .flush()
            .expect("flushing log file");
    }
}
