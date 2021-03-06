#![feature(plugin)]
#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::config::{Config, Environment};
use rocket::response::status::BadRequest;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate libc;
extern crate rand;
extern crate unix_socket;

use serde_json::value::Value;

mod mpv;

#[get("/ping")]
fn ping() -> &'static str {
    "Pong!"
}

#[derive(Deserialize)]
struct EnqueueData {
    uris: Vec<String>,
    replace: Option<String>,
}

#[post("/enqueue", data = "<params>")]
fn enqueue(
    params: Json<EnqueueData>,
    command_adapter: State<mpv::CommandAdapter>,
) -> Result<(), ()> {
    let replace = params
        .replace
        .as_ref()
        .map(|s| s == "true")
        .unwrap_or(false);

    let replace = if replace { "replace" } else { "append-play" };

    for uri in &params.uris {
        let cmd = ["loadfile", &*uri, replace];
        command_adapter.send(&cmd).map_err(|_| ())?; // TODO
    }
    Ok(())
}

#[derive(Deserialize)]
struct CommandData {
    no_wait: bool,
    args: Vec<String>,
}

#[post("/command", data = "<params>")]
fn command(
    params: Json<CommandData>,
    command_adapter: State<mpv::CommandAdapter>,
) -> Result<JsonValue, BadRequest<JsonValue>> {
    let args = params.args.iter().map(|s| &**s).collect::<Vec<_>>();
    let error = |v| BadRequest(Some(json!({ "error": v })));
    if params.no_wait {
        command_adapter
            .send(&args)
            .map(|rid| json!({ "request_id": rid }))
            .map_err(|err| error(format!("{:?}", err)))
    } else {
        let resp = command_adapter
            .send_recv::<Value>(&args)
            .map_err(|err| error(format!("{:?}", err)))?;
        println!("{:?}", resp);

        if let Some(data) = resp.data {
            Ok(data.into())
        } else {
            let err_str = resp
                .error
                .expect("unsuccessful response without error field");
            Err(error(err_str))
        }
    }
}

fn main() {
    println!("serving on :9922"); // TODO
    let cmd = mpv::new_command_adapter();

    {
        let args = ["get_property", "fullscreen"];
        println!(
            "property fullscreen: {:?}",
            cmd.send_recv::<bool>(&args).unwrap().data.unwrap()
        );
    }

    let config = Config::build(Environment::Development)
        .port(9922)
        .finalize()
        .unwrap();
    rocket::custom(config)
        .manage(cmd)
        .mount("/", routes![ping, enqueue, command])
        .launch();
}
