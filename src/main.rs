use std::sync::atomic::Ordering;
use std::sync::Arc;

use loom::sync::atomic::AtomicBool;
use loom::sync::atomic::AtomicUsize;
use loom::thread;

fn main() {
    loom::model(|| {
        let once = Arc::new(AtomicBool::new(false));
        let value = Arc::new(AtomicUsize::new(0));

        let a = loom_thread_test(&value, &once);
        let b = loom_thread_test(&value, &once);
        let c = loom_thread_test(&value, &once);

        let _ = a.join();
        let _ = b.join();
        let _ = c.join();

        assert_eq!(value.load(Ordering::Relaxed), 1);
    });
}

fn loom_thread_test(value: &Arc<AtomicUsize>, once: &Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let value = value.clone();
    let once = once.clone();

    thread::spawn(move || {
        if let Ok(_) = once.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed) {
            value.fetch_add(1, Ordering::Relaxed);
        }
    })
}
