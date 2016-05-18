extern crate iron;
#[macro_use]
extern crate mime;
extern crate syncbox;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate urlencoded;


use iron::prelude::*;
use iron::status;
use iron::typemap::Key;

use syncbox::LinkedQueue;
use std::process::Command;
use std::thread;
use std::ops::Deref;
use urlencoded::UrlEncodedQuery;

fn spawn_mpv(media: String) {
    println!("$ mpv {}", media);
    match Command::new("mpv").arg(media).status() {
        Err(e) => println!("failed to execute process: {}", e),
        Ok(status) => println!("process exited with: {}", status),
    }
}


fn spawn_player_thread(queue: LinkedQueue<Box<String>>) {
    thread::spawn(move || {
        // TODO: spawn and control mpv with ipc
        loop {
            let media = queue.take();
            println!("playing {}...", media);
            spawn_mpv(*media);
            println!("waiting for queue...")
        }
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
