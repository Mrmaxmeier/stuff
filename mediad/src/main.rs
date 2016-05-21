#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde_json;

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


fn main() {
    let tx = mpv::spawn_player_thread();
    let cmd = mpv::new_command_adapter(tx);
    cmd.send(vec!["get_property".to_owned(), "time-pos".to_owned()]);

    let router = router!(get "/ping" => ping, post "/enqueue" => enqueue);

    let mut chain = Chain::new(router);
    // chain.link(persistent::Read::<CommandAdapter>::both());
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
        // let arc = req.get::<persistent::Read<CommandAdapter>>().unwrap();
        // let queue = arc.deref();
        // for uri in uris {
        // queue.put(Box::new(uri.to_owned()));
        // }
        Ok(Response::with((status::Ok, "")))
    }
}
