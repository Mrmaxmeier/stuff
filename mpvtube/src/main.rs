extern crate iron;

use iron::prelude::*;
use iron::method::Post;
use iron::mime::Mime;
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

		let content_type = "application/json".parse::<Mime>().unwrap();
		Ok(Response::with((content_type, status::Ok, "{}")))

	}

	println!("serving on :9922");
	Iron::new(handle_request).http("localhost:9922").unwrap();
}
