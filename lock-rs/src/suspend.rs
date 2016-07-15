use std::time::SystemTime;
use std::process::Command;

pub struct Suspender {}

impl Suspender {
    pub fn new() -> Suspender {
        Suspender {}
    }

    pub fn suspend(&mut self) {
        let now = SystemTime::now();
        Command::new("systemctl")
            .arg("suspend")
            .output()
            .expect("failed to suspend");
        ::std::thread::sleep(::std::time::Duration::from_secs(3));
        println!("elapsed while hibernating: {:?}", now.elapsed());
    }
}
