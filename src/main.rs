use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Acquire;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::Release;
use std::sync::Arc;

use loom::sync::atomic::AtomicBool;
use loom::sync::atomic::AtomicUsize;
use loom::thread;

#[derive(Default, Clone)]
struct BadSpinlock(Arc<AtomicBool>);

impl BadSpinlock {
    fn new() -> Self {
        Self::default()
    }

    fn lock(&self) {
        while self.0.compare_exchange_weak(false, true, Relaxed, Relaxed).is_err() {
            // no-op
        }
    }

    fn unlock(&self) {
        self.0.store(false, Relaxed);
    }
}

fn main() {
    loom::model(|| {
        let once = Arc::new(AtomicBool::new(false));
        let value = Arc::new(AtomicUsize::new(0));

        let a = loom_thread_test(&value, &once);
        let b = loom_thread_test(&value, &once);

        let _ = a.join();
        let _ = b.join();

        assert_eq!(value.load(Relaxed), 1);

        let lock = BadSpinlock::new();

        let mut val = 0u8;
        let val_ref: *mut u8 = &mut val;

        spinlock_test(&lock, val_ref);
        spinlock_test(&lock, val_ref);

        // should panic
        assert_eq!(unsafe { *val_ref }, 2);
    });
}

fn spinlock_test(lock: &BadSpinlock, val: *mut u8) -> thread::JoinHandle<()> {
    let lock = lock.clone();

    thread::spawn(move || {
        lock.lock();
        unsafe { *val += 1 }
        lock.unlock();
    })
}

fn loom_thread_test(value: &Arc<AtomicUsize>, once: &Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let value = value.clone();
    let once = once.clone();

    thread::spawn(move || {
        if let Ok(_) = once.compare_exchange(false, true, Relaxed, Relaxed) {
            value.fetch_add(1, Relaxed);
        }
    })
}
