use std::process;
use libc;

pub struct I3Lock {
    process: Option<process::Child>,
}

impl I3Lock {
    pub fn new() -> I3Lock {
        I3Lock { process: None }
    }

    fn active(&self) -> bool {
        match self.process {
            None => false,
            Some(ref child) => unsafe { libc::signal(child.id() as i32, 0) == 0 },
        }
    }

    pub fn ensure_locked(&mut self) {
        println!("locker active: {:?}", self.active());
        if !self.active() {
            let mut cmd = process::Command::new("i3lock");
            cmd.arg("-n");
            self.process = Some(cmd.spawn().unwrap());
            println!("locked.");
        }
    }
}
