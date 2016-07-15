use x11::xss;
use x11::xss::XScreenSaverInfo;
use x11::xlib;

use std::time::Duration;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::sync::mpsc;
use std::thread;

use SockCommand;

pub struct XIdleService {
    display: *mut xlib::Display,
    root: xlib::Drawable,
    pub lock_threshold: Duration,
    pub sleep_threshold: Duration,
}

impl XIdleService {
    pub fn new() -> XIdleService {
        let (display, root) = unsafe {
            let display = xlib::XOpenDisplay(null());
            if display == null_mut() {
                panic!("can't open display");
            };
            let root = xlib::XRootWindow(display, xlib::XDefaultScreen(display));
            (display, root)
        };

        XIdleService {
            display: display,
            root: root,
            lock_threshold: Duration::from_secs(60 * 3),
            sleep_threshold: Duration::from_secs(60 * 10),
        }
    }

    pub fn query(&mut self) -> XScreenSaverInfo {
        unsafe {
            let mut info = zeroed();
            xss::XScreenSaverQueryInfo(self.display, self.root, &mut info);
            info
        }
    }

    pub fn idle(&mut self) -> Duration {
        Duration::from_millis(self.query().idle)
    }

    pub fn notify(&mut self, tx: mpsc::Sender<SockCommand>) {
        loop {
            let idle = self.idle();
            if idle >= self.lock_threshold {
                tx.send(SockCommand::Lock).unwrap();
            }
            if idle >= self.sleep_threshold {
                tx.send(SockCommand::Suspend).unwrap();
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}

impl Drop for XIdleService {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.display) };
    }
}
