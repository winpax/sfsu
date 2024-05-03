use std::{
    fs::File,
    io::Write,
    sync::{atomic::AtomicUsize, Arc},
};

use log::{Level, LevelFilter};
use sprinkles::{eprintln_red, eprintln_yellow};

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

    pub async fn cleanup_logs() -> anyhow::Result<()> {
        use tokio::task::JoinSet;

        let logging_dir = sprinkles::Scoop::logging_dir()?;

        let mut logs = tokio::fs::read_dir(logging_dir).await?;
        let mut set = JoinSet::new();

        let cleaned = Arc::new(AtomicUsize::new(0));
        while let Some(entry) = logs.next_entry().await? {
            let cleaned = cleaned.clone();
            set.spawn(async move {
                use std::sync::atomic::Ordering;

                let cleaned_value = cleaned.load(Ordering::Relaxed);
                if cleaned_value > 10 {
                    return Ok(()); // Don't do anything if we've cleaned up enough
                }

                tokio::fs::remove_file(entry.path()).await?;

                cleaned.store(cleaned_value + 1, Ordering::Relaxed);

                Ok::<_, tokio::io::Error>(())
            });
        }

        while let Some(result) = set.join_next().await {
            result??;
        }

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
