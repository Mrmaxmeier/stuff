use std;
use std::process::Command;
use std::thread;
use std::sync::mpsc;
use unix_socket::UnixStream;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use serde_json;
use serde::de::Deserialize;
use rand::{thread_rng, Rng};
use std::error::Error;
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


pub fn spawn_player_thread(adapter: CommandAdapter) {
    let adapter2 = adapter.clone();
    thread::spawn(move || {
        let pid = unsafe { libc::getpid() };
        // FIXME: does this belong to /var/run?
        loop {
            let (tx, rx) = mpsc::channel::<String>();
            {
                let mut mpv_tx = (*adapter.mpv_tx).lock().unwrap();
                *mpv_tx = Some(tx);
            }

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
                        if let Err(e) = stream.write_all(buf) {
                            println!("write_all error {:?}", e);
                            break;
                        }
                    }
                });
            };

            {
                let adapter = adapter.clone();
                thread::spawn(move || {
                    let f = BufReader::new(stream);
                    for line in f.lines() {
                        let line = line.unwrap();
                        println!("mpv-ipc< {}", line);
                        let deserialized: GenericMPVResponse = serde_json::from_str(&line).unwrap();
                        println!("{:?}", deserialized);
                        if let Some(request_id) = deserialized.request_id {
                            let req_handlers = (*adapter.req_handlers).lock().unwrap();
                            if let Some(tx) = req_handlers.get(&request_id) {
                                tx.send(line).unwrap()
                            } else {
                                println!("missing handler for request_id {}!", request_id);
                            }
                        }
                    }
                });
            };

            match cmd.status() {
                Err(e) => println!("failed to execute process: {}", e),
                Ok(status) => println!("process exited with: {}", status),
            }
        }
    });
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));
        if adapter2.mpv_tx.lock().unwrap().is_some() {
            break
        }
    }
}

type CallbackHashmap = HashMap<usize, mpsc::Sender<String>>;

#[derive(Clone)]
pub struct CommandAdapter {
    req_handlers: Arc<Mutex<CallbackHashmap>>,
    next_req_id: Arc<AtomicUsize>,
    mpv_tx: Arc<Mutex<Option<mpsc::Sender<String>>>>,
}


impl CommandAdapter {
    pub fn send(&mut self, cmd_args: Vec<&str>) -> Result<usize, Box<Error>> {
        let cmd_args = cmd_args.iter().map(|a: &&str| (*a).to_owned()).collect::<Vec<_>>();
        let req_id = self.next_req_id.fetch_add(1, Ordering::SeqCst);
        let cmd = MPVCommand {
            command: cmd_args,
            request_id: req_id,
        };
        let serialized = try!(serde_json::to_string(&cmd));
        try!(self.tx_send(serialized));
        Ok(req_id)
    }

    fn tx_send(&self, s: String) -> Result<(), mpsc::SendError<String>> {
        let tx = self.mpv_tx.lock().unwrap();
        tx.clone().unwrap().send(s)
    }

    pub fn send_recv<T: Deserialize>(&mut self,
                                     cmd_args: Vec<&str>)
                                     -> Result<MPVResponse<T>, Box<Error>> {
        let req_id = try!(self.send(cmd_args));
        let (tx, rx) = mpsc::channel::<String>();
        {
            let mut req_handlers = (*self.req_handlers).lock().unwrap();
            req_handlers.insert(req_id, tx);
        }
        println!("waiting for reqid {}", req_id);
        let line = try!(rx.recv());
        let deserialized: MPVResponse<T> = serde_json::from_str(&line).unwrap();
        Ok(deserialized)
    }
}


pub fn new_command_adapter() -> CommandAdapter {
    let adapter = CommandAdapter {
        mpv_tx: Arc::new(Mutex::new(None)),
        next_req_id: Arc::new(AtomicUsize::new(0)),
        req_handlers: Arc::new(Mutex::new(HashMap::new())),
    };
    spawn_player_thread(adapter.clone());
    adapter
}


#[derive(Serialize, Deserialize, Debug)]
struct MPVCommand {
    command: Vec<String>,
    request_id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericMPVResponse {
    error: Option<String>,
    request_id: Option<usize>,
    event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MPVResponse<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    request_id: usize,
}
