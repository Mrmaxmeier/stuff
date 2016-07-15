#![feature(custom_derive, plugin)]
#![plugin(docopt_macros)]

extern crate libc;
extern crate x11;
extern crate rustc_serialize;
extern crate docopt;
extern crate notify_rust;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate pbr;

use std::thread;
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::prelude::*;
use num::FromPrimitive;

use pbr::ProgressBar;

use std::sync::mpsc;
use notify_rust::Notification;

mod xidle;
mod i3lock;
mod suspend;

docopt!(Args derive Debug, "
lock, a simple lock-state-manager.

Usage:
  lock daemon [--progress]
  lock lock
  lock suspend
  lock (-h | --help)

Options:
  -h --help     Show this screen.
  -v --verbose  Display verbose logs.
");

const SOCKFILE: &'static str = "/tmp/lock.sock";

enum_from_primitive! {
    #[derive(Debug)]
    pub enum SockCommand {
        Lock = 0,
        Suspend = 1,
        Quit = 2
    }
}

fn daemon(progress: bool) {
    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || {
            listen(tx).unwrap();
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || xidle::XIdleService::new().notify(tx));
    }

    if progress {
        thread::spawn(|| {
            let mut service = xidle::XIdleService::new();

            fn mk_pb(service: &xidle::XIdleService) -> ProgressBar {
                let mut pb = ProgressBar::new(service.lock_threshold.as_secs());
                pb.show_speed = false;
                pb.show_time_left = false;
                pb
            }

            let mut pb = mk_pb(&service);
            let mut pb_progress = 0;
            pb.inc();
            pb.tick();
            loop {
                let idle = service.idle();
                //println!("idle: {:?}", service.idle());
                //println!("till lock: {:?}", service.lock_threshold - service.idle());
                if idle.as_secs() > pb_progress {
                    pb_progress = pb.inc()
                } else if pb_progress > idle.as_secs() + 1 {
                    pb_progress = 0;
                    pb = mk_pb(&service);
                    pb.tick();
                }
                thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    let mut locker = i3lock::I3Lock::new();
    let mut suspender = suspend::Suspender::new();

    println!("daemoning");
    for cmd in rx.iter() {
        match cmd {
            SockCommand::Quit => {
                println!("bye-bye");
                return
            },
            SockCommand::Lock => locker.ensure_locked(),
            SockCommand::Suspend => {
                println!("suspending...");
                suspender.suspend()
            }
        }
    }
}

fn listen(tx: mpsc::Sender<SockCommand>) -> Result<(), std::io::Error> {

    if std::fs::remove_file(SOCKFILE).is_ok() {
        println!("removed old socket");
    }

    let listener = try!(UnixListener::bind(SOCKFILE));

    for stream in listener.incoming() {
        let stream = try!(stream);
        let tx = tx.clone();
        thread::spawn(move || {
            for byte in stream.bytes() {
                println!("recvd byte: {:?}", byte);
                if let Ok(byte) = byte {
                    if let Some(cmd) = SockCommand::from_u8(byte) {
                        tx.send(cmd).unwrap()
                    }
                }
            }
        });
    }

    drop(listener);
    Ok(())
}

fn send(command: SockCommand) -> Result<(), std::io::Error> {
    let mut stream = try!(UnixStream::connect(SOCKFILE));
    stream.write_all(&[command as u8])
}


fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    if args.cmd_daemon {
        if send(SockCommand::Quit).is_ok() {
            println!("stopped other daemon...");
        }
        daemon(args.flag_progress)
    } else if args.cmd_lock {
        Notification::new()
            .summary("locking remotely...")
            .show().unwrap();
        send(SockCommand::Lock).unwrap()
    } else if args.cmd_suspend {
        Notification::new()
            .summary("suspending remotely...")
            .show().unwrap();
        send(SockCommand::Suspend).unwrap()
    }
}
