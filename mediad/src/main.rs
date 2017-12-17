#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::State;
use rocket::config::{Config, Environment};
use rocket::request::{FromForm, FormItems};
use rocket::response::status::BadRequest;
use rocket_contrib::Json;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate unix_socket;
extern crate rand;
extern crate libc;

use serde_json::value::Value;

mod mpv;


#[get("/ping")]
fn ping() -> &'static str {
    "Pong!"
}

#[derive(FromForm)]
struct EnqueueData {
    uri: String,
    replace: Option<String>
}

#[post("/enqueue?<params>")]
fn enqueue(params: EnqueueData, command_adapter: State<mpv::CommandAdapter>) -> Result<(), ()> {
    let replace = match params.replace.map(|s| s == "true") {
        Some(true) => "replace",
        _          => "append-play"
    };

    for uri in vec![params.uri] { // TODO: vector
        let cmd = vec!["loadfile", &*uri, replace];
        command_adapter.send(cmd).map_err(|_| ())?; // TODO
    }
    Ok(())
}

struct CommandData {
    no_wait: bool,
    args: Vec<String>,
}

impl<'f> FromForm<'f> for CommandData {
    // In practice, we'd use a more descriptive error type.
    type Error = ();

    fn from_form(items: &mut FormItems<'f>, strict: bool) -> Result<CommandData, ()> {
        let mut no_wait = false;
        let mut args = Vec::new();

        for (key, value) in items {
            match key.as_str() {
                "arg" => {
                    let decoded = value.url_decode().map_err(|_| ())?;
                    args.push(decoded);
                },
                "no-wait" => no_wait = true,
                _ if strict => return Err(()),
                _ => { /* allow extra value when not strict */ }
            }
        }

        Ok(CommandData { no_wait, args })
    }
}

#[post("/command?<params>")]
fn command(params: CommandData, command_adapter: State<mpv::CommandAdapter>) -> Result<Json<Value>, BadRequest<Json<Value>>> {
    let args = params.args.iter().map(|s| &**s).collect();
    let error = |v|  BadRequest(Some(Json(json!({"error": v}))));
    if params.no_wait {
        command_adapter.send(args)
            .map(|rid| Json(json!({
                "request_id": rid
            })))
            .map_err(|err| error(format!("{:?}", err)))
    } else {
        let resp = command_adapter.send_recv::<Value>(args)
            .map_err(|err| error(format!("{:?}", err)))?;
        println!("{:?}", resp);


        if let Some(data) = resp.data {
            Ok(Json(data))
        } else {
            let err_str = resp.error.expect("unsuccessful response without error field");
            Err(error(err_str))
        }
    }
}

fn main() {
    println!("serving on :9922"); // TODO
    let cmd = mpv::new_command_adapter();

    {
        let args = vec!["get_property", "fullscreen"];
        println!("property fullscreen: {:?}",
                 cmd.send_recv::<bool>(args).unwrap().data.unwrap());
    }

    let config = Config::build(Environment::Development)
        .port(9922)
        .finalize().unwrap();
    rocket::custom(config, true)
        .manage(cmd)
        .mount("/", routes![ping, enqueue, command])
        .launch();
}