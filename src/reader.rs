//! Bounded, read-only background work. A newer queued request supersedes an older one.
//! Dropping the owner signals shutdown without waiting for a blocked filesystem call.
use std::sync::{Arc, Condvar, Mutex};

struct State<Q, R> {
    pending: Option<(u64, Q)>,
    result: Option<(u64, R)>,
    stopped: bool,
}

pub(crate) struct LatestReader<Q, R> {
    shared: Arc<(Mutex<State<Q, R>>, Condvar)>,
}

impl<Q, R> std::fmt::Debug for LatestReader<Q, R> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LatestReader")
            .finish_non_exhaustive()
    }
}

impl<Q: Send + 'static, R: Send + 'static> LatestReader<Q, R> {
    pub(crate) fn new(
        name: &str,
        mut read: impl FnMut(Q) -> R + Send + 'static,
    ) -> std::io::Result<Self> {
        let shared = Arc::new((
            Mutex::new(State {
                pending: None,
                result: None,
                stopped: false,
            }),
            Condvar::new(),
        ));
        let worker = Arc::clone(&shared);
        std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                loop {
                    let (lock, ready) = &*worker;
                    let request = {
                        let mut state =
                            lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                        while state.pending.is_none() && !state.stopped {
                            state = ready
                                .wait(state)
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                        }
                        if state.stopped {
                            return;
                        }
                        state.pending.take().expect("pending read")
                    };
                    let result = read(request.1);
                    let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if state.stopped {
                        return;
                    }
                    state.result = Some((request.0, result));
                }
            })?;
        Ok(Self { shared })
    }
    pub(crate) fn submit(&self, generation: u64, request: Q) {
        let (lock, ready) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending = Some((generation, request));
        state.result = None;
        ready.notify_one();
    }
    pub(crate) fn take_result(&self) -> Option<(u64, R)> {
        self.shared
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .result
            .take()
    }
    pub(crate) fn clear_pending(&self) {
        let mut state = self
            .shared
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending = None;
        state.result = None;
    }
}

impl<Q, R> Drop for LatestReader<Q, R> {
    fn drop(&mut self) {
        let (lock, ready) = &*self.shared;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.stopped = true;
        state.pending = None;
        state.result = None;
        ready.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_slow_read_does_not_block_submission_or_drop() {
        let reader = LatestReader::new("slow-read-test", |value| {
            std::thread::sleep(Duration::from_millis(100));
            value
        })
        .unwrap();
        let started = Instant::now();
        reader.submit(1, 42);
        assert!(
            started.elapsed() < Duration::from_millis(20),
            "submission waited for filesystem work"
        );
        drop(reader);
        assert!(
            started.elapsed() < Duration::from_millis(40),
            "reader drop waited for an in-flight read"
        );
    }

    #[test]
    fn pending_requests_and_completed_results_remain_bounded_to_latest() {
        let (started, observed) = std::sync::mpsc::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        let reader = LatestReader::new("latest-read-test", move |value| {
            started.send(value).unwrap();
            if value == 1 {
                blocked.recv().unwrap();
            }
            value
        })
        .unwrap();
        reader.submit(1, 1);
        assert_eq!(observed.recv_timeout(Duration::from_secs(2)).unwrap(), 1);
        for value in 2..=10_000 {
            reader.submit(value, value);
        }
        release.send(()).unwrap();
        assert_eq!(
            observed.recv_timeout(Duration::from_secs(2)).unwrap(),
            10_000
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some((generation, value)) = reader.take_result()
                && generation == 10_000
            {
                assert_eq!(value, 10_000);
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(
            observed.try_recv().is_err(),
            "superseded requests were executed"
        );
        assert!(reader.take_result().is_none());
    }
}
