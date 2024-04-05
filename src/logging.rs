use std::{fs::File, io::Write};

use log::LevelFilter;

pub mod panics;

pub struct Logger {
    file: File,
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Self {
        Self::from_file(sfsu::Scoop::new_log().expect("new log"), verbose)
    }

    pub fn from_file(file: File, verbose: bool) -> Self {
        Self { file, verbose }
    }

    pub fn init(verbose: bool) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(Logger::new(verbose)))?;
        log::set_max_level(LevelFilter::Trace);

        debug!("Initialized logger");

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if self.verbose {
            true
        } else {
            metadata.level() < log::Level::Debug
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            // TODO: Add a queue of sorts because this doesn't work well with multiple threads
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
