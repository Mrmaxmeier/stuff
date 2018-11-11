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
        Command::new("systemctl")
            .arg("suspend")
            .output()
            .expect("failed to suspend");

        // https://github.com/systemd/systemd/blob/36376e0b71d97e276429e0e6307f116587ac83bd/TODO#L440-L443
        thread::sleep(Duration::from_secs(2));

        println!("elapsed while suspended: {:?}", now.elapsed());
        self.last_resume = SystemTime::now();
    }
}
