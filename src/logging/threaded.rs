use std::{io::Write, sync::atomic::AtomicBool};

pub static THREADED_LOGGER: Logger = Logger {
    locked: AtomicBool::new(false),
};

#[derive(Debug)]
pub struct Logger {
    locked: AtomicBool,
}

impl Logger {
    pub fn try_lock(&self) -> Option<LoggerGuard<'_>> {
        if self.locked.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }

        self.locked
            .store(true, std::sync::atomic::Ordering::Relaxed);

        Some(LoggerGuard {
            stdout: std::io::stderr(),
            logger: self,
        })
    }

    pub fn lock(&self) -> LoggerGuard<'_> {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
        }
    }

    pub(self) fn unlock(&self) {
        self.locked
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct LoggerGuard<'a> {
    stdout: std::io::Stderr,
    logger: &'a Logger,
}

impl<'a> Write for LoggerGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.stdout.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.stdout.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stdout.write_fmt(fmt)
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl<'a> Drop for LoggerGuard<'a> {
    fn drop(&mut self) {
        self.logger.unlock();
    }
}
