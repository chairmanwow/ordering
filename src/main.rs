use std::sync::atomic::Ordering;
use std::sync::Arc;

use loom::sync::atomic::AtomicBool;
use loom::sync::atomic::AtomicUsize;
use loom::thread;

fn main() {
    loom::model(|| {
        let once_bool = Arc::new(AtomicBool::new(false));
        let value = Arc::new(AtomicUsize::new(0));

        let once_bool_ = once_bool.clone();
        let value_ = value.clone();

        thread::spawn(move || {
            if let Ok(_) =
                once_bool.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            {
                fetch_and_test(&value);
            }
        });

        thread::spawn(move || {
            if let Ok(_) =
                once_bool_.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            {
                fetch_and_test(&value_);
            }
        });
    });
}

fn fetch_and_test(v: &AtomicUsize) {
    v.fetch_add(1, Ordering::Relaxed);
    assert_eq!(v.load(Ordering::Relaxed), 1);
}
