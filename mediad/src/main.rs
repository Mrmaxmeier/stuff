extern crate iron;
#[macro_use]
extern crate mime;
extern crate syncbox;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate urlencoded;
extern crate unix_socket;


use iron::prelude::*;
use iron::status;
use iron::typemap::Key;

use syncbox::LinkedQueue;
use std::process::Command;
use std::thread;
use std::ops::Deref;
use urlencoded::UrlEncodedQuery;
use unix_socket::UnixStream;
use std::io::prelude::*;

fn poll_connect(path: &str) -> UnixStream {
	loop {
		match UnixStream::connect(path) {
			Ok(stream) => return stream,
			Err(e) => println!("{}", e),
		}
		std::thread::sleep(std::time::Duration::from_secs(1));
	}
}


fn spawn_player_thread(queue: LinkedQueue<Box<String>>) {
    thread::spawn(move || {
		let path = "/tmp/mpv-sock-1337";
		let mut cmd = Command::new("mpv");
		cmd.arg("--input-ipc-server");
		cmd.arg(path);
		cmd.arg("--idle");
		cmd.spawn().unwrap();
		let mut stream = poll_connect(path);
		println!("connected to mpv ipc.");
        //stream.write_all(&[0u8; 0]);
        //let mut buf = vec![];
        //println!("stream: {:?}", stream.read_to_end(&mut buf));
		//println!("stream: {:?}", buf);
        loop {
			let mut s = String::new();
			println!("ipc: {:?}", stream.read_to_string(&mut s));
			println!("ipc: {:?}", s);
            let media = queue.take();
            println!("playing {}...", media);
            println!("waiting for queue...")
        }
	    match cmd.status() {
	        Err(e) => println!("failed to execute process: {}", e),
	        Ok(status) => println!("process exited with: {}", status),
	    }
        drop(stream);
    });
}


macro_rules! try_or_return {
    ($res:expr, $orelse:expr) => {{
        match $res {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(e) => {
                return $orelse(e)
            }
        }
    }};
}

macro_rules! get_or_return {
    ($res:expr, $orelse:expr) => {{
        match $res {
            ::std::option::Option::Some(val) => val,
            ::std::option::Option::None => {
                return $orelse()
            }
        }
    }};
}



#[derive(Copy, Clone)]
pub struct MediaQueue;

impl Key for MediaQueue {
    type Value = LinkedQueue<Box<String>>;
}

fn main() {
    let queue: LinkedQueue<Box<String>> = LinkedQueue::new();
    spawn_player_thread(queue.clone());


    let router = router!(get "/ping" => ping, post "/enqueue" => enqueue);

    let mut chain = Chain::new(router);
    chain.link(persistent::Read::<MediaQueue>::both(queue));
    println!("serving on :9922");
    Iron::new(chain).http("localhost:9922").unwrap();

    fn ping(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Pong!")))
    }

    fn enqueue(req: &mut Request) -> IronResult<Response> {
        let hashmap = try_or_return!(req.get::<UrlEncodedQuery>(),
                                     |_| Ok(Response::with((status::BadRequest, ""))));
        let uris = get_or_return!(hashmap.get("uri"),
                                  || Ok(Response::with((status::BadRequest, ""))));
        let arc = req.get::<persistent::Read<MediaQueue>>().unwrap();
        let queue = arc.deref();
        for uri in uris {
            queue.put(Box::new(uri.to_owned()));
        }
        Ok(Response::with((status::Ok, "")))
    }
}
