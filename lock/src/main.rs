extern crate libc;
extern crate notify_rust;
extern crate term;
extern crate term_size;
extern crate x11;
#[macro_use]
extern crate clap;

use notify_rust::Notification;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::thread;

mod i3lock;
mod status;
mod suspend;
mod xidle;

const SOCKFILE: &str = "/tmp/lock.sock";

#[derive(Debug)]
pub enum SockCommand {
    Lock = 0,
    Suspend = 1,
    Quit = 2,
}

impl SockCommand {
    fn from_u8(v: u8) -> Option<SockCommand> {
        use self::SockCommand::*;
        match v {
            0 => Some(Lock),
            1 => Some(Suspend),
            2 => Some(Quit),
            _ => None,
        }
    }
}

fn daemon(progress: bool) {
    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || listen(&tx).unwrap());
    }

    {
        let tx = tx.clone();
        thread::spawn(move || xidle::XIdleService::new().notify(&tx));
    }

    if progress {
        thread::spawn(|| {
            let mut service = xidle::XIdleService::new();
            let mut progress = status::ProgressBar::new(service.sleep_threshold);
            progress.render().unwrap();
            loop {
                let idle = service.idle();
                //println!("idle: {:?}", service.idle());
                //println!("till lock: {:?}", service.lock_threshold - service.idle());
                progress.current = idle;
                progress.render().unwrap();
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
                return;
            }
            SockCommand::Lock => locker.ensure_locked(),
            SockCommand::Suspend => {
                println!("suspending...");
                suspender.suspend()
            }
        }
    }
}

fn listen(tx: &mpsc::Sender<SockCommand>) -> Result<(), std::io::Error> {
    if std::fs::remove_file(SOCKFILE).is_ok() {
        println!("removed old socket");
    }

    let listener = UnixListener::bind(SOCKFILE)?;

    for stream in listener.incoming() {
        let stream = stream?;
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
    let mut stream = UnixStream::connect(SOCKFILE)?;
    stream.write_all(&[command as u8])
}

fn main() {
    let matches = clap_app!(lock =>
        (version: "1.0")
        (about: "lock, a simple lock-state-manager")
        (settings: &[clap::AppSettings::SubcommandRequired])
        (@subcommand lock => (about: "Locks the current session"))
        (@subcommand suspend => (about: "Suspends the current session"))
        (@subcommand daemon =>
            (about: "Runs the lock daemon")
            (@arg progress: -p --progress "Displays a cli-hud containing the current state"))
    )
    .get_matches();

    match matches.subcommand() {
        ("daemon", Some(options)) => {
            if send(SockCommand::Quit).is_ok() {
                println!("stopped other daemon...");
            }
            daemon(options.is_present("progress"))
        }
        ("lock", _) => {
            Notification::new()
                .summary("locking remotely...")
                .show()
                .unwrap();
            send(SockCommand::Lock).unwrap()
        }
        ("suspend", _) => {
            Notification::new()
                .summary("suspending remotely...")
                .show()
                .unwrap();
            send(SockCommand::Suspend).unwrap()
        }
        _ => unreachable!(),
    }
}
