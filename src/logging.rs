use std::{fs::File, io::Write};

use chrono::Local;
use log::{Level, LevelFilter};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sprinkles::contexts::ScoopContext;

use crate::output::colours::{eprintln_red, eprintln_yellow};

pub mod panics;

pub struct Logger {
    file: Option<File>,
    verbose: bool,
}

#[allow(dead_code)]
impl Logger {
    const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;

    pub async fn new<C>(ctx: &impl ScoopContext<C>, verbose: bool) -> Self {
        let file = async move {
            let logs_dir = ctx.logging_dir()?;
            let date = Local::now();
            let log_file = async {
                let mut i = 0;
                loop {
                    i += 1;

                    let log_path =
                        logs_dir.join(format!("sfsu-{}-{i}.log", date.format("%Y-%m-%d-%H-%M-%S")));

                    if !log_path.exists() {
                        break File::create(log_path);
                    }
                }
            };
            let timeout = async {
                use std::time::Duration;
                use tokio::time;

                time::sleep(Duration::from_secs(5)).await;
            };
            let log_file = tokio::select! {
                res = log_file => anyhow::Ok(res),
                () = timeout => anyhow::bail!("Timeout creating new log"),
            }??;

            anyhow::Ok(log_file)
        }
        .await
        .ok();

        Self::from_file(file, verbose)
    }

    pub fn from_file(file: Option<File>, verbose: bool) -> Self {
        Self { file, verbose }
    }

    pub async fn init<C>(
        ctx: &impl ScoopContext<C>,
        verbose: bool,
    ) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(Logger::new(ctx, verbose).await))?;
        log::set_max_level(Self::LEVEL_FILTER);

        debug!("Initialized logger");

        Ok(())
    }

    pub fn cleanup_logs<C>(ctx: &impl ScoopContext<C>) -> anyhow::Result<()> {
        let logging_dir = ctx.logging_dir()?;

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
                    writeln!(
                        self.file.as_ref().unwrap(),
                        "{}: {}",
                        record.level(),
                        record.args()
                    )
                    .expect("writing to log file");
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = self.file.as_ref() {
            file.try_clone()
                .expect("cloning log file")
                .flush()
                .expect("flushing log file");
        }
    }
}
