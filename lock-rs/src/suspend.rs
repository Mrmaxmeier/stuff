use std::time::Instant;
use std::process::Command;

pub struct Suspender {}

impl Suspender {
    pub fn new() -> Suspender {
        Suspender {}
    }

    pub fn suspend(&mut self) {
        let now = Instant::now();
        Command::new("systemctl")
            .arg("suspend")
            .output()
            .expect("failed to suspend");
        println!("elapsed: {:?}", now.elapsed());
    }
}
