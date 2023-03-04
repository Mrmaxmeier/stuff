use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

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
            println!(
                "refusing to suspend due to recent resume ({} secs ago)",
                diff.as_secs()
            );
            return;
        }

        self.trigger_suspend();


        // https://github.com/systemd/systemd/blob/36376e0b71d97e276429e0e6307f116587ac83bd/TODO#L440-L443
        thread::sleep(Duration::from_secs(2));

        println!("elapsed while suspended: {:?}", now.elapsed());
        self.last_resume = SystemTime::now();
    }

    fn trigger_suspend(&self) {
        if self.allow_hibernation() {
            let res = Command::new("systemctl")
                .arg("suspend-then-hibernate")
                .output()
                .expect("failed to spawn systemctl suspend-then-hibernate");

            // suspend-then-hibernate might fail in low-swap situations
            if res.status.success() {
                return;
            }
        }

        Command::new("systemctl")
                .arg("suspend")
                .output()
                .expect("failed to spawn systemctl suspend");
    }

    fn allow_hibernation(&self) -> bool {
        let hostname = std::fs::read_to_string("/etc/hostname").unwrap_or("unknown".into());
        hostname.trim() !=  "tuna"
    }
}
