use std::time::{Duration,SystemTime};
use std::process::Command;

pub struct Suspender {
    last_resume: SystemTime,
}

impl Suspender {
    pub fn new() -> Suspender {
        Suspender {
            last_resume: SystemTime::now(),
        }
    }

    pub fn suspend(&mut self) {
        let now = SystemTime::now();
        let diff = now.duration_since(self.last_resume).unwrap();
        if diff < Duration::from_secs(30) {
            println!("refusing to suspend due to recent resume ({} secs ago)", diff.as_secs());
            return
        }
        Command::new("systemctl")
            .arg("suspend")
            .output()
            .expect("failed to suspend");
        println!("elapsed while suspended: {:?}", now.elapsed());
        self.last_resume = SystemTime::now();
    }
}
