use std::{
    future::Future,
    sync::Arc,
    task::Poll,
    thread,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

pub enum WaitResult {
    Ready,
    Pending(Duration),
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    rate: u64,
    completed: Arc<Mutex<u64>>,
    reset: Duration,
    remaining: Arc<Mutex<Duration>>,
    now: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(rate: u64, reset: Duration) -> Self {
        Self {
            rate,
            completed: Arc::new(Mutex::new(rate)),
            reset,
            remaining: Arc::new(Mutex::new(reset)),
            now: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn try_wait(&self) -> WaitResult {
        let mut completed = self.completed.lock();
        if let Some(new_completed) = completed.checked_sub(1) {
            *completed = new_completed;
            return WaitResult::Ready;
        }

        *completed = self.rate;

        let delta = self.now.lock().elapsed();
        *self.now.lock() = Instant::now();

        if let Some(remaining) = self.remaining.lock().checked_sub(delta) {
            *self.remaining.lock() = remaining;
            WaitResult::Pending(remaining)
        } else {
            *self.remaining.lock() = self.reset;
            WaitResult::Ready
        }
    }

    pub async fn wait(&self) {
        RateLimitWait {
            limiter: self.clone(),
        }
        .await;
    }
}

struct RateLimitWait {
    limiter: RateLimiter,
}

impl Future for RateLimitWait {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        #[cfg(feature = "fake_feature")]
        {
            if let Some(completed) = self.limiter.completed.lock().checked_sub(1) {
                *self.limiter.completed.lock() = completed;
                return Poll::Ready(());
            }

            *self.limiter.completed.lock() = self.limiter.rate;

            let delta = self.limiter.now.lock().elapsed();
            *self.limiter.now.lock() = Instant::now();

            if let Some(remaining) = self.limiter.remaining.lock().checked_sub(delta) {
                let waker = cx.waker().clone();
                thread::spawn(move || {
                    thread::sleep(remaining);
                    waker.wake();
                });

                *self.limiter.remaining.lock() = remaining;
                Poll::Pending
            } else {
                *self.limiter.remaining.lock() = self.limiter.reset;
                Poll::Ready(())
            }
        }

        match self.limiter.try_wait() {
            WaitResult::Ready => Poll::Ready(()),
            WaitResult::Pending(remaining) => {
                let waker = cx.waker().clone();
                thread::spawn(move || {
                    thread::sleep(remaining);
                    waker.wake();
                });

                Poll::Pending
            }
        }
    }
}
