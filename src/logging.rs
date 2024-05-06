use std::{fs::File, io::Write};

use log::{Level, LevelFilter};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sprinkles::contexts::ScoopContext;

use crate::output::colours::{eprintln_red, eprintln_yellow};

pub mod panics;

pub struct Logger {
    file: File,
    verbose: bool,
}

#[allow(dead_code)]
impl Logger {
    const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;

    pub async fn new(verbose: bool) -> Self {
        Self::from_file(sprinkles::Scoop::new_log().await.expect("new log"), verbose)
    }

    pub fn new_sync(verbose: bool) -> Self {
        Self::from_file(sprinkles::Scoop::new_log_sync().expect("new log"), verbose)
    }

    pub fn from_file(file: File, verbose: bool) -> Self {
        Self { file, verbose }
    }

    pub async fn init(verbose: bool) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(Logger::new(verbose).await))?;
        log::set_max_level(Self::LEVEL_FILTER);

        debug!("Initialized logger");

        Ok(())
    }

    pub fn init_sync(verbose: bool) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(Logger::new_sync(verbose)))?;
        log::set_max_level(Self::LEVEL_FILTER);

        debug!("Initialized logger");

        Ok(())
    }

    pub fn cleanup_logs() -> anyhow::Result<()> {
        let logging_dir = sprinkles::Scoop::logging_dir()?;

        let logs = std::fs::read_dir(logging_dir)?.collect::<Result<Vec<_>, _>>()?;

        logs.into_iter()
            .rev()
            .skip(10)
            .par_bridge()
            .try_for_each(|entry| std::fs::remove_file(entry.path()))?;

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if self.verbose {
            true
        } else {
            metadata.level() < Level::Debug
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            match record.metadata().level() {
                Level::Error => eprintln_red!("{}", record.args()),
                Level::Warn => eprintln_yellow!("{}", record.args()),
                _ => {
                    // TODO: Add a queue of sorts because this doesn't work well with multiple threads
                    writeln!(&self.file, "{}: {}", record.level(), record.args())
                        .expect("writing to log file");
                }
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
