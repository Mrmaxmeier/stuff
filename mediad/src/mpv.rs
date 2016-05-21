use std;
use std::process::Command;
use std::thread;
use std::sync::mpsc;
use unix_socket::UnixStream;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use serde_json;
use rand::{thread_rng, Rng};
use libc;

fn poll_connect(path: &str) -> UnixStream {
    std::thread::sleep(std::time::Duration::from_millis(250));
    loop {
        match UnixStream::connect(path) {
            Ok(stream) => return stream,
            Err(e) => println!("{}", e),
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}


pub fn spawn_player_thread() -> mpsc::Sender<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let pid = unsafe {
            libc::getpid()
        };
        // FIXME: does this belong to /var/run?
        let path = &*format!("/tmp/mpv-sock-{}-{}", pid, thread_rng().gen::<u16>());
        let mut cmd = Command::new("mpv");
        cmd.arg("--input-ipc-server");
        cmd.arg(path);
        cmd.arg("--idle");
        cmd.spawn().unwrap();
        let stream = poll_connect(path);
        println!("connected to mpv ipc socket ({})", path);
        {
            let mut stream = stream.try_clone().unwrap();
            thread::spawn(move || {
                for line in (&rx).iter() {
                    println!("mpv-ipc> {}", line);
                    let line = line + "\n";
                    let buf: &[u8] = &line.into_bytes();
                    stream.write_all(buf).unwrap();
                }
            })
        };

        let f = BufReader::new(stream);
        for line in f.lines() {
            let line = line.unwrap();
            println!("mpv-ipc< {}", line);
            let deserialized: MPVResponse<String> = serde_json::from_str(&line).unwrap();
            println!("{:?}", deserialized);
        }
        match cmd.status() {
            Err(e) => println!("failed to execute process: {}", e),
            Ok(status) => println!("process exited with: {}", status),
        }
    });
    tx
}


#[derive(Clone)]
pub struct CommandAdapter {
    req_handlers: HashMap<usize, mpsc::Sender<String>>,
    next_req_id: Arc<AtomicUsize>,
    mpv_tx: mpsc::Sender<String>,
}


impl CommandAdapter {
    pub fn send(mut self, cmd_args: Vec<String>) -> String {
        let req_id = self.next_req_id.fetch_add(1, Ordering::SeqCst);
        let cmd = MPVCommand {
            command: cmd_args,
            request_id: req_id,
        };
        let serialized = serde_json::to_string(&cmd).unwrap();
        self.mpv_tx.send(serialized).unwrap();
        let (tx, rx) = mpsc::channel::<String>();
        self.req_handlers.insert(req_id, tx);
        rx.recv().unwrap()
    }
}


pub fn new_command_adapter(tx: mpsc::Sender<String>) -> CommandAdapter {
    CommandAdapter {
        mpv_tx: tx,
        next_req_id: Arc::new(AtomicUsize::new(0)),
        req_handlers: HashMap::new(),
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct MPVCommand {
    command: Vec<String>,
    request_id: usize,
}
// { "command": ["get_property", "time-pos"], "request_id": 100 }

#[derive(Serialize, Deserialize, Debug)]
struct MPVResponse<T> {
    data: Option<T>,
    error: String,
    request_id: usize,
}
// { "error": "success", "data": 1.468135, "request_id": 100 }
