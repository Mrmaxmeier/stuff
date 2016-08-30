#![feature(custom_derive, plugin)]
#![plugin(docopt_macros)]
#![plugin(serde_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate hyper;
extern crate url;
extern crate serde;
extern crate serde_json;

use hyper::client::Client;
use hyper::status::StatusCode;
use url::Url;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

docopt!(Args derive Debug, "
mediad-client.

Usage:
  mdc ping
  mdc queue [--replace] <uri>
  mdc pause [--toggle]
  mdc raw [--no-response] <args>...
  mdc playlist [--all]
  mdc restart
  mdc (-h | --help)

Options:
  -h --help     Show this screen.
  -v --verbose  Display verbose logs.
");


#[derive(Debug, PartialEq, Deserialize)]
struct PlaylistEntry {
    filename: String,
    playing: Option<bool>,
    current: Option<bool>,
}

enum Commands {
    Ping,
    Queue(bool, String),
    Pause(bool),
    Raw(bool, Vec<String>),
    Playlist,
    Restart,
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

fn mpv_cmd(url_base: &Url, args: Vec<&str>) -> String {
    let mut url = url_base.clone();
    url.set_path("command");
    {
        let mut query_pairs = url.query_pairs_mut();
        for arg in args {
            query_pairs.append_pair("arg", arg);
        }
    }
    let mut response = send(url);
    // println!("{:?}", response);
    let mut s = String::new();
    response.read_to_string(&mut s).unwrap();
    s
}

fn mpv_cmd_nr(url_base: &Url, args: Vec<&str>) -> Result<(), ()> {
    let mut url = url_base.clone();
    url.set_path("command");
    {
        let mut query_pairs = url.query_pairs_mut();
        for arg in args {
            query_pairs.append_pair("arg", arg);
        }
    }
    url.query_pairs_mut().append_pair("no-wait", "true");
    let mut response = send(url);
    // println!("{:?}", response);
    let mut s = String::new();
    response.read_to_string(&mut s).unwrap();
    Ok(())
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
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let command = if args.cmd_ping {
        Commands::Ping
    } else if args.cmd_queue {
        Commands::Queue(args.flag_replace, args.arg_uri)
    } else if args.cmd_pause {
        Commands::Pause(args.flag_toggle)
    } else if args.cmd_raw {
        Commands::Raw(args.flag_no_response, args.arg_args)
    } else if args.cmd_playlist {
        Commands::Playlist
    } else if args.cmd_restart {
        Commands::Restart
    } else {
        unreachable!()
    };


    let mut url = Url::parse("http://localhost:9922").unwrap();
    match command {
        Commands::Ping => {
            url.set_path("ping");
            let client = Client::new();
            std::process::exit(match client.get(url).send() {
                Ok(result) => {
                    match result.status {
                        StatusCode::Ok => 0,
                        _ => 1,
                    }
                }
                Err(_) => 1,
            });
        }
        Commands::Pause(toggle) => {
            url.set_path("pause");
            url.query_pairs_mut().append_pair("toggle", &format!("{}", toggle));
            send(url);
        }
        Commands::Queue(replace, uri) => {
            url.set_path("enqueue");
            url.query_pairs_mut().append_pair("replace", &format!("{}", replace));
            let uri = valid_file_path(&uri).unwrap_or_else(|| uri);
            url.query_pairs_mut().append_pair("uri", &uri);
            send(url);
        }
        Commands::Raw(no_response, args) => {
            url.set_path("command");
            {
                let mut query_pairs = url.query_pairs_mut();
                for arg in args {
                    query_pairs.append_pair("arg", &arg);
                }
                if no_response {
                    query_pairs.append_pair("no-wait", "1");
                }
            }
            let mut response = send(url);
            println!("{:?}", response);
            let mut s = String::new();
            println!("{:?}", response.read_to_string(&mut s));
            println!("{}", s);
        }
        Commands::Playlist => {
            let data = mpv_cmd(&url, vec!["get_property", "playlist"]);

            let playlist: Vec<PlaylistEntry> = serde_json::from_str(&data).unwrap();
            let mut found_current = false;
            for entry in playlist {
                if let Some(true) = entry.playing {
                    print!("|");
                }
                if let Some(true) = entry.current {
                    print!("> ");
                    found_current = true;
                }
                if found_current || args.flag_all {
                    println!("{}", entry.filename);
                }
            }
        }
        Commands::Restart => {
            let playlist_data = mpv_cmd(&url, vec!["get_property", "playlist"]);
            let playlist: Vec<PlaylistEntry> = serde_json::from_str(&playlist_data).unwrap();
            let time: f64 = mpv_cmd(&url, vec!["get_property", "time-pos"]).parse().unwrap();
            println!("playlist: {:#?}", playlist);
            println!("time current: {}", time);
            println!("quitting mpv...");
            mpv_cmd_nr(&url, vec!["quit"]).unwrap();
            sleep(Duration::from_secs(1));
            let mut found_current = false;
            for entry in &playlist {
                let is_current = entry.current.unwrap_or(false);
                found_current |= is_current;

                if found_current {
                    println!("queueing {:?}", entry.filename);
                    mpv_cmd(&url, vec!["loadfile", &*entry.filename, "append-play"]);
                    sleep(Duration::from_secs(2));
                }

                if is_current {
                    println!("waiting for file load...");
                    sleep(Duration::from_secs(5)); // TODO wait for actual events
                    println!("seeking to {:?}", time);
                    mpv_cmd(&url, vec!["seek", &*format!("{}", time), "absolute+keyframes"]);
                    sleep(Duration::from_secs(2));
                }
            }
        }
    }
}
