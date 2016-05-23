#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde_json;
extern crate serde;

extern crate iron;
#[macro_use]
extern crate mime;
extern crate syncbox;
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

mod mpv;

macro_rules! try_or_return {
    ($res:expr, $orelse:expr) => {{
        match $res {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(e) => {
                println!("{:?}", e);
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

    let router = router!(get "/ping" => ping, post "/enqueue" => enqueue);

    let mut chain = Chain::new(router);
    chain.link(persistent::Write::<CommandAdapterState>::both(cmd));
    println!("serving on :9922");
    Iron::new(chain).http("localhost:9922").unwrap();

    fn ping(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Pong!")))
    }

    fn enqueue(req: &mut Request) -> IronResult<Response> {
        let hashmap = try_or_return!(req.get::<UrlEncodedQuery>(),
                                     |_| Ok(Response::with((status::BadRequest, "invalid query"))));
        let uris = get_or_return!(hashmap.get("uri"), || {
            Ok(Response::with((status::BadRequest, "missing uri parameter")))
        });
        let replace = match hashmap.get("replace") {
            Some(_) => "replace",
            None => "append-play",
        };
        let mutex = req.get::<persistent::Write<CommandAdapterState>>().unwrap();
        let mut guard = mutex.lock().unwrap();
        let adapter = &mut *guard;
        for uri in uris {
            let cmd = vec!["loadfile", uri, replace];
            try_or_return!(adapter.send(cmd),
                           |e| Ok(Response::with((status::BadRequest, format!("{:?}", e)))));
        }
        Ok(Response::with((status::Ok, "")))
    }
}
