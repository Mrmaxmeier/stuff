#![feature(custom_derive, plugin)]
#![plugin(docopt_macros)]

extern crate libc;
extern crate x11;
extern crate rustc_serialize;
extern crate docopt;
extern crate notify_rust;

use std::thread;
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::prelude::*;

use std::sync::mpsc;
use notify_rust::Notification;

mod xidle;

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

#[derive(Debug)]
pub enum SockCommand {
    Lock = 0,
    Suspend = 1,
    Quit = 2,
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
            loop {
                println!("idle: {:?}", service.idle());
                thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    println!("daemoning");
    for cmd in rx.iter() {
        println!("got cmd {:?}", cmd);
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
                tx.send(SockCommand::Quit).unwrap() // TODO
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
            .show()
            .unwrap();
        send(SockCommand::Lock).unwrap()
    } else if args.cmd_suspend {
        Notification::new()
            .summary("suspending remotely...")
            .show()
            .unwrap();
        send(SockCommand::Suspend).unwrap()
    }
}
