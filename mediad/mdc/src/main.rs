use clap::Parser;
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
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

fn send(url: Url) -> Response {
    let client = Client::new();
    let result = client.post(url).send().unwrap();
    match result.status() {
        StatusCode::OK => (),
        _ => println!("Error: {:?}", result.status()),
    };
    result
}

fn send_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(endpoint: &str, data: T) -> R {
    let client = Client::new();
    let url = Url::parse(&*format!("http://localhost:9922/{}", endpoint)).unwrap();
    let result = client.post(url).json(&data).send().unwrap();
    serde_json::from_reader(result).unwrap()
}

#[derive(Serialize)]
struct CommandData {
    no_wait: bool,
    args: Vec<String>,
}

fn mpv_cmd<T: serde::de::DeserializeOwned, V: AsRef<str>>(args: &[V]) -> T {
    send_json(
        "command",
        CommandData {
            no_wait: false,
            args: args.iter().map(|s| s.as_ref().to_owned()).collect(),
        },
    )
}

fn mpv_cmd_nr<T: AsRef<str>>(args: &[T]) {
    send_json::<CommandData, serde_json::Value>(
        "command",
        CommandData {
            no_wait: true,
            args: args.iter().map(|s| s.as_ref().to_owned()).collect(),
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

#[derive(Parser)]
enum Opts {
    /// Checks connectivity
    Ping,
    /// Queues an entry into mpv's playlist
    Queue {
        /// Replaces currently playing file with INPUT
        #[clap(short, long)]
        replace: bool,
        input: String,
    },
    /// Pauses playback
    Pause {
        #[clap(short, long)]
        toggle: bool,
    },
    /// Restart mpv and restore previous state
    Restart,
    /// Display mpv's current playlist
    Playlist {
        /// Include past entries
        #[clap(short, long)]
        all: bool,
    },
    /// Send a raw RPC command to mpv
    Raw {
        /// Return without waiting for a response
        #[clap(short, long)]
        no_response: bool,
        arguments: Vec<String>,
    },
}

fn main() {
    let mut url = Url::parse("http://localhost:9922").unwrap();
    let opts = Opts::parse();
    match opts {
        Opts::Ping => {
            url.set_path("ping");
            let client = Client::new();
            std::process::exit(match client.get(url).send() {
                Ok(result) => match result.status() {
                    StatusCode::OK => 0,
                    _ => 1,
                },
                Err(_) => 1,
            });
        }
        Opts::Pause { toggle } => {
            url.set_path("pause");
            url.query_pairs_mut()
                .append_pair("toggle", &format!("{}", toggle));
            send(url);
        }
        Opts::Queue { replace, input } => {
            let uri = valid_file_path(&input).unwrap_or(input);
            let mode = if replace { "replace" } else { "append-play" };
            mpv_cmd_nr(&["loadfile", &uri, mode]);
        }
        Opts::Raw {
            no_response,
            arguments,
        } => {
            if no_response {
                mpv_cmd_nr(&arguments);
            } else {
                let res: serde_json::Value = mpv_cmd(&arguments);
                let out = serde_json::to_string_pretty(&res).unwrap();
                println!("{}", out);
            }
        }
        Opts::Playlist { all } => {
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
                if found_current || all {
                    println!("{}", entry.filename);
                }
            }
        }
        Opts::Restart => {
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
    }
}
