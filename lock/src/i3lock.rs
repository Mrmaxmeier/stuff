use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

pub struct I3Lock {
    active: Arc<AtomicBool>,
}

impl I3Lock {
    pub fn new() -> I3Lock {
        I3Lock {
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn ensure_locked(&mut self) {
        if let Ok(_) = self.active.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            let active = self.active.clone();
            thread::spawn(move || {
                process::Command::new("killall")
                    .arg("-SIGUSR1")
                    .arg("dunst")
                    .output()
                    .unwrap();
                let now = SystemTime::now();
                process::Command::new("i3lock")
                    .arg("-n")
                    .arg("-c")
                    .arg("A6E22E")
                    .output()
                    .unwrap();
                active.store(false, Ordering::SeqCst);
                println!("unlocked after {:?}", now.elapsed());
                process::Command::new("killall")
                    .arg("-SIGUSR2")
                    .arg("dunst")
                    .output()
                    .unwrap();
            });
            println!("locked.");
        }
    }
}
