#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate hyper;
extern crate url;

use hyper::client::Client;
use hyper::status::StatusCode;
use url::Url;

docopt!(Args derive Debug, "
mediad-client.

Usage:
  mdc queue <uri>
  mdc pause [--toggle]
  mdc restart
  mdc (-h | --help)

Options:
  -h --help     Show this screen.
  -v --verbose  Display verbose logs.
");

fn send(url: Url) {
    let client = Client::new();
    let result = client.post(url)
                       .send()
                       .unwrap();
    match result.status {
        StatusCode::Ok => println!("Ok"),
        _   => println!("Error: {:?}", result.status),
    }
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut url = Url::parse("http://localhost:9922").unwrap();
    if args.cmd_queue {
        println!("enqueueing '{}'...", args.arg_uri);
        url.set_path("enqueue");
        url.query_pairs_mut().append_pair("uri", &*args.arg_uri);
    } else if args.cmd_pause {
        println!("pausing...");
        url.set_path("pause");
        url.query_pairs_mut().append_pair("toggle", &*format!("{}", args.flag_toggle));
    } else if args.cmd_restart {
        println!("restarting...");
        url.set_path("restart");
    }
    send(url);
}
