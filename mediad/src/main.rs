#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate iron;
#[macro_use]
extern crate mime;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate urlencoded;
extern crate unix_socket;
extern crate rand;
extern crate libc;

use urlencoded::UrlEncodedQuery;
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use serde_json::value::Value;

mod mpv;

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

fn bad_request<T: std::fmt::Debug>(e: T) -> iron::IronResult<iron::Response> {
    println!("{:?}", e);
    let err_str = format!("{:?}", e);
    let resp = Response::with((status::BadRequest, err_str));
    Ok(resp)
}

#[derive(Copy, Clone)]
pub struct CommandAdapterState;

impl Key for CommandAdapterState {
    type Value = mpv::CommandAdapter;
}

fn main() {
    let cmd = mpv::new_command_adapter();

    {
        let mut cmd = cmd.clone();
        let args = vec!["get_property", "fullscreen"];
        println!("property fullscreen: {:?}",
                 cmd.send_recv::<bool>(args).unwrap().data.unwrap());
    }

    let router = router!(
        ping:    get  "/ping"    => ping,
        enqueue: post "/enqueue" => enqueue,
        command: post "/command" => command,
    );

    let mut chain = Chain::new(router);
    chain.link(persistent::Write::<CommandAdapterState>::both(cmd));
    println!("serving on :9922");
    Iron::new(chain).http("localhost:9922").unwrap();

    fn ping(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Pong!")))
    }

    fn enqueue(req: &mut Request) -> IronResult<Response> {
        let hashmap = try_or_return!(req.get::<UrlEncodedQuery>(), bad_request);
        let uris = get_or_return!(hashmap.get("uri"), || bad_request("invalid url"));
        let replace = match hashmap.get("replace") {
            Some(vec) if vec.get(0) == Some(&"true".to_owned()) => "replace",
            _ => "append-play",
        };
        let mutex = req.get::<persistent::Write<CommandAdapterState>>().unwrap();
        let mut guard = mutex.lock().unwrap();
        let adapter = &mut *guard;
        for uri in uris {
            let cmd = vec!["loadfile", uri, replace];
            try_or_return!(adapter.send(cmd), bad_request);
        }
        Ok(Response::with((status::Ok, "")))
    }

    fn command(req: &mut Request) -> IronResult<Response> {
        let hashmap = try_or_return!(req.get::<UrlEncodedQuery>(), bad_request);
        let no_wait = match hashmap.get("no-wait") {
            Some(_) => true,
            None => false,
        };
        let args = get_or_return!(hashmap.get("arg"), || bad_request("missing arg parameter"));
        let args: Vec<&str> = args.iter().map(|s| &**s).collect();

        let mutex = req.get::<persistent::Write<CommandAdapterState>>().unwrap();
        let mut guard = mutex.lock().unwrap();
        let adapter = &mut *guard; // TODO

        if no_wait {
            try_or_return!(adapter.send(args), bad_request);
            Ok(Response::with((status::Ok, "")))
        } else {
            let resp = try_or_return!(adapter.send_recv::<Value>(args), bad_request);
            let parsed = try_or_return!(resp.into(), bad_request);
            let reserialized = try_or_return!(serde_json::to_string(&parsed), bad_request);
            Ok(Response::with((mime!(Application / Json), status::Ok, reserialized)))
        }
    }
}
