extern crate iron;
#[macro_use]
extern crate mime;

use iron::prelude::*;
use iron::method::Post;
use iron::status;
use std::io::Read;
use std::process::Command;

fn spawn_mpv(media: String) {
	println!("$ mpv {}", media);
	match Command::new("mpv").arg(media).status() {
		Err(e) => println!("failed to execute process: {}", e),
		Ok(status) => println!("process exited with: {}", status)
	}
}

fn main() {
	fn handle_request(req: &mut Request) -> IronResult<Response> {
		if req.method != Post {
			return Ok(Response::with((status::NotFound, "invalid method")))
		}
		if req.url.path[0] != "submit" {
			println!("invalid url: {}", req.url);
			return Ok(Response::with((status::BadRequest, "invalid url")))
		}

		let mut buffer = String::new();
		req.body.read_to_string(&mut buffer).unwrap();
		println!("{}", buffer);
		spawn_mpv(buffer);

		Ok(Response::with((mime!(Application/Json), status::Ok, "{}")))
	}

	println!("serving on :9922");
	Iron::new(handle_request).http("localhost:9922").unwrap();
}
