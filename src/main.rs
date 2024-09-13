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
            println!("yield");
            // so that loom doesn't freak out
            thread::yield_now();
        }
        println!("go");
    }

    fn unlock(&self) {
        self.0.store(false, Relaxed);
    }
}

fn main() {
    loom::model(|| {
        // let once = Arc::new(AtomicBool::new(false));
        // let value = Arc::new(AtomicUsize::new(0));
        //
        // let a = loom_thread_test(&value, &once);
        // let b = loom_thread_test(&value, &once);
        //
        // let _ = a.join();
        // let _ = b.join();
        //
        // assert_eq!(value.load(Relaxed), 1);

        let lock = BadSpinlock::new();

        let val: *mut usize = &mut 0usize;

        let a = spinlock_test(&lock, val);
        let b = spinlock_test(&lock, val);
        let _ = a.join();
        let _ = b.join();
        assert_eq!(unsafe { *val }, 2);

        // let data = Default::default();
        // let is_ready = Arc::new(AtomicBool::new(false));
        //
        // let a = bad_thread_write(&data, &is_ready);
        // let b = bad_thread_read(&data, &is_ready);
        //
        // let _ = a.join();
        // let _ = b.join();

        // let x = Default::default();
        // let y = Default::default();
        // let a = classic_example_loader(&x, &y);
        // let b = classic_example_storer(&x, &y);
        //
        // let _ = a.join();
        // let _ = b.join();
    });
}

fn classic_example_loader(a: &Arc<AtomicUsize>, b: &Arc<AtomicUsize>) -> thread::JoinHandle<()> {
    let a = a.clone();
    let b = b.clone();

    thread::spawn(move || {
        a.store(1, Relaxed);
        b.store(2, Relaxed);
    })
}
fn classic_example_storer(a: &Arc<AtomicUsize>, b: &Arc<AtomicUsize>) -> thread::JoinHandle<()> {
    let a = a.clone();
    let b = b.clone();

    thread::spawn(move || {
        let b = b.load(Relaxed);
        let a = a.load(Relaxed);

        assert!((a == 0 && b == 0) || (a == 1 && b == 0) || (a == 1 && b == 2))
    })
}

fn spinlock_test(lock: &BadSpinlock, val: *mut usize) -> thread::JoinHandle<()> {
    let lock = lock.clone();

    thread::spawn(move || {
        lock.lock();
        unsafe { *val += 1 };
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

fn bad_thread_write(data: &Arc<AtomicUsize>, is_ready: &Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let is_ready = is_ready.clone();
    let data = data.clone();

    thread::spawn(move || {
        data.store(123, Relaxed);
        is_ready.store(true, Relaxed);
    })
}

fn bad_thread_read(data: &Arc<AtomicUsize>, is_ready: &Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let is_ready = is_ready.clone();
    let data = data.clone();

    thread::spawn(move || {
        while !is_ready.load(Relaxed) {
            thread::yield_now()
        }

        assert_eq!(data.load(Relaxed), 123);
    })
}
