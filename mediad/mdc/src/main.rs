extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate hyper;
extern crate url;
#[macro_use]
extern crate clap;

use hyper::client::Client;
use hyper::header::ContentType;
use hyper::status::StatusCode;
use std::io::prelude::*;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

#[derive(Debug, PartialEq, Deserialize)]
struct PlaylistEntry {
    filename: String,
    playing: Option<bool>,
    current: Option<bool>,
}

fn send(url: Url) -> hyper::client::Response {
    let client = Client::new();
    let result = client.post(url).send().unwrap();
    match result.status {
        StatusCode::Ok => (),
        _ => println!("Error: {:?}", result.status),
    };
    result
}

fn send_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(endpoint: &str, data: T) -> R {
    let client = Client::new();
    let body = serde_json::to_string(&data).unwrap();

    let url = Url::parse(&*format!("http://localhost:9922/{}", endpoint)).unwrap();
    let result = client
        .post(url)
        .header(ContentType::json())
        .body(&body)
        .send()
        .unwrap();
    serde_json::from_reader(result).unwrap()
}

#[derive(Serialize)]
struct CommandData {
    no_wait: bool,
    args: Vec<String>,
}

fn mpv_cmd<T: serde::de::DeserializeOwned>(args: &[&str]) -> T {
    send_json(
        "command",
        CommandData {
            no_wait: false,
            args: args.iter().map(|&s| s.to_owned()).collect(),
        },
    )
}

fn mpv_cmd_nr(args: &[&str]) {
    send_json::<CommandData, serde_json::Value>(
        "command",
        CommandData {
            no_wait: true,
            args: args.iter().map(|&s| s.to_owned()).collect(),
        },
    );
}

fn valid_file_path(path_str: &str) -> Option<String> {
    let path = Path::new(&path_str);
    if path.is_file() {
        if let Ok(buf) = path.canonicalize() {
            return buf.to_str().map(|s| s.into());
        }
    }
    None
}

fn main() {
    let matches = clap_app!(mdc =>
        (version: "1.0")
        (about: "A cli for mediad")
        (settings: &[clap::AppSettings::SubcommandRequired])
        (@subcommand ping => (about: "Checks connectivity"))
        (@subcommand queue =>
            (about: "Queues an entry into the playlist")
            (@arg replace: -r --replace "Replaces current playing file with the new queued one")
            (@arg INPUT: +required "The URI"))
        (@subcommand pause =>
            (about: "Pauses the current playback")
            (@arg toggle: -t --toggle "Toggles playing state instead of setting it to paused"))
        (@subcommand restart => (about: "Restarts mediad and restores the previous state"))
        (@subcommand playlist =>
            (about: "Displays the current playlist")
            (@arg all: -a --all "Includes played entries"))
        (@subcommand raw =>
            (about: "Sends a raw command to mpv")
            (@arg no_response: -n --no-response "Returns immediately")
            (@arg INPUT: +required +multiple "Command args"))
    ).get_matches();

    let mut url = Url::parse("http://localhost:9922").unwrap();
    match matches.subcommand() {
        ("ping", _) => {
            url.set_path("ping");
            let client = Client::new();
            std::process::exit(match client.get(url).send() {
                Ok(result) => match result.status {
                    StatusCode::Ok => 0,
                    _ => 1,
                },
                Err(_) => 1,
            });
        }
        ("pause", Some(options)) => {
            url.set_path("pause");
            let toggle = options.is_present("toggle");
            url.query_pairs_mut()
                .append_pair("toggle", &format!("{}", toggle));
            send(url);
        }
        ("queue", Some(options)) => {
            url.set_path("enqueue");
            let replace = options.is_present("replace");
            url.query_pairs_mut()
                .append_pair("replace", &format!("{}", replace));
            let uri = options.value_of("INPUT").unwrap();
            let uri = valid_file_path(uri).unwrap_or_else(|| uri.into());
            url.query_pairs_mut().append_pair("uri", &uri);
            send(url);
        }
        ("raw", Some(options)) => {
            url.set_path("command");
            {
                let mut query_pairs = url.query_pairs_mut();
                for arg in options.values_of("INPUT").unwrap() {
                    query_pairs.append_pair("arg", arg);
                }
                if options.is_present("no_response") {
                    query_pairs.append_pair("no-wait", "1");
                }
            }
            let mut response = send(url);
            println!("{:#?}", response);
            let mut s = String::new();
            println!("{:?}", response.read_to_string(&mut s));
            println!("{}", s);
        }
        ("playlist", Some(options)) => {
            let playlist: Vec<PlaylistEntry> = mpv_cmd(&["get_property", "playlist"]);

            let mut found_current = false;
            for entry in playlist {
                if let Some(true) = entry.playing {
                    print!("|");
                }
                if let Some(true) = entry.current {
                    print!("> ");
                    found_current = true;
                }
                if found_current || options.is_present("all") {
                    println!("{}", entry.filename);
                }
            }
        }
        ("restart", _) => {
            let playlist: Vec<PlaylistEntry> = mpv_cmd(&["get_property", "playlist"]);
            let time: f64 = mpv_cmd(&["get_property", "time-pos"]);
            println!("playlist: {:#?}", playlist);
            println!("time current: {}", time);
            println!("quitting mpv...");
            mpv_cmd_nr(&["quit"]);
            sleep(Duration::from_secs(1));
            let mut found_current = false;
            for entry in &playlist {
                let is_current = entry.current.unwrap_or(false);
                found_current |= is_current;

                if found_current {
                    println!("queueing {:?}", entry.filename);
                    let _: serde_json::Value =
                        mpv_cmd(&["loadfile", &*entry.filename, "append-play"]);
                    sleep(Duration::from_secs(2));
                }

                if is_current {
                    println!("waiting for file load...");
                    sleep(Duration::from_secs(5)); // TODO wait for actual events
                    println!("seeking to {:?}", time);
                    let _: serde_json::Value =
                        mpv_cmd(&["seek", &*format!("{}", time), "absolute+keyframes"]);
                    sleep(Duration::from_secs(2));
                }
            }
        }
        _ => panic!("invalid subcommand"),
    }
}
