#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate hyper;
extern crate url;

use hyper::client::Client;
use hyper::status::StatusCode;
use url::Url;
use std::io::prelude::*;

docopt!(Args derive Debug, "
mediad-client.

Usage:
  mdc ping
  mdc queue [--replace] <uri>
  mdc pause [--toggle]
  mdc raw [--no-response] <args>...
  mdc restart
  mdc (-h | --help)

Options:
  -h --help     Show this screen.
  -v --verbose  Display verbose logs.
");

fn send<F>(url: Url, handler: Option<F>)
    where F: FnOnce(hyper::client::Response)
{
    let client = Client::new();
    let result = client.post(url).send().unwrap();
    match result.status {
        StatusCode::Ok => (),
        _ => println!("Error: {:?}", result.status),
    }
    if let Some(handler) = handler {
        handler(result);
    }
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut url = Url::parse("http://localhost:9922").unwrap();
    let handler: Option<_> = if args.cmd_ping {
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
    } else if args.cmd_queue {
        url.set_path("enqueue");
        url.query_pairs_mut().append_pair("uri", &args.arg_uri);
        None
    } else if args.cmd_pause {
        url.set_path("pause");
        url.query_pairs_mut().append_pair("toggle", &format!("{}", args.flag_toggle));
        if args.flag_replace {
            url.query_pairs_mut().append_pair("replace", "true");
        };
        None
    } else if args.cmd_raw {
        url.set_path("command");
        let mut query_pairs = url.query_pairs_mut();
        for arg in args.arg_args {
            query_pairs.append_pair("arg", &arg);
        }
        if args.flag_no_response {
            query_pairs.append_pair("no-wait", "1");
        }
        Some(|mut response: hyper::client::Response| {
            println!("{:?}", response);
            let mut s = String::new();
            println!("{:?}", response.read_to_string(&mut s));
            println!("{}", s);
        })
    } else if args.cmd_restart {
        url.set_path("restart");
        None
    } else {
        unreachable!{};
    };

    send(url, handler);
}
