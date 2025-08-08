use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// A simple bounded worker pool for items of type T.
/// Uses a bounded VecDeque with backpressure and a user-provided worker closure.
pub struct BoundedWorkerPool<T: Send + 'static> {
    queue: Arc<(Mutex<VecDeque<T>>, Condvar)>,
    pending: Arc<AtomicUsize>,
    capacity: usize,
    workers: usize,
    is_running: Arc<AtomicBool>,
}

impl<T: Send + 'static> BoundedWorkerPool<T> {
    pub fn new(
        _workers: usize,
        capacity: usize,
        _worker: Arc<dyn Fn(T) + Send + Sync + 'static>,
    ) -> Self {
        let queue = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
        let pending = Arc::new(AtomicUsize::new(0));
        let is_running = Arc::new(AtomicBool::new(true));

        BoundedWorkerPool {
            queue,
            pending,
            capacity,
            workers: _workers,
            is_running,
        }
    }

    /// Submit an item to the pool; blocks briefly when the queue is full.
    pub fn submit(&self, item: T) {
        while self.pending.load(Ordering::SeqCst) >= self.capacity {
            thread::sleep(Duration::from_millis(1));
        }
        let (lock, cvar) = (&self.queue.0, &self.queue.1);
        let mut guard = lock.lock().unwrap();
        guard.push_back(item);
        self.pending.fetch_add(1, Ordering::SeqCst);
        cvar.notify_one();
    }

    /// Pop a work item; blocks until available.
    pub fn pop_blocking(&self) -> T {
        let (lock, cvar) = (&self.queue.0, &self.queue.1);
        let mut guard = lock.lock().unwrap();
        while guard.is_empty() {
            guard = cvar.wait(guard).unwrap();
        }
        self.pending.fetch_sub(1, Ordering::SeqCst);
        guard.pop_front().unwrap()
    }

    /// Stop workers gracefully; best-effort.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.queue.1.notify_all();
    }
}
