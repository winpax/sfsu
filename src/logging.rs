use std::{fs::File, io::Write};

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
        let logger = Box::new(Logger::new(verbose));
        let logger = Box::leak(logger);

        log::set_logger(logger)?;

        debug!("Initialized logger");

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if self.verbose {
            true
        } else {
            metadata.level() > log::Level::Trace
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            if record.metadata().level() == log::Level::Trace {
                eprintln!("{}: {}", record.level(), record.args());
            } else {
                writeln!(&self.file, "{}: {}", record.level(), record.args())
                    .expect("writing to log file");
            }
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
