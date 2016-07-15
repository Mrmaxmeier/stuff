use std::process;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

pub struct I3Lock {
    active: Arc<AtomicBool>,
}

impl I3Lock {
    pub fn new() -> I3Lock {
        I3Lock {
            active: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn ensure_locked(&mut self) {
        if !self.active.compare_and_swap(false, true, Ordering::SeqCst) {
            let active = self.active.clone();
            thread::spawn(move || {
                let now = SystemTime::now();
                process::Command::new("i3lock")
                    .arg("-n")
                    .output().unwrap();
                active.store(false, Ordering::SeqCst);
                println!("unlocked after {:?}", now.elapsed());
            });
            println!("locked.");
        }
    }
}
