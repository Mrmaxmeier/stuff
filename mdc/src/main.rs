#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate hyper;
extern crate url;

use hyper::client::Client;
use url::Url;

docopt!(Args derive Debug, "
mediad-client.

Usage:
  mdc queue <uri>
  mdc pause [--toggle]
  mdc restart
  mdc (-h | --help)
  mdc --version

Options:
  -h --help     Show this screen.
  -V --version  Show version.
  -v --verbose  Display verbose logs.
  --port=<kn>   Server port. [default: 9922]
");
// FIXME set port [default: 9922]


fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    let port = 9922; //args.arg_port.or_else(9922);
    if args.cmd_queue {
        println!("enqueueing '{}'...", args.arg_uri);
        let client = Client::new();
        let mut url = Url::parse("http://localhost/enqueue").unwrap();
        url.query_pairs_mut().append_pair("uri", &*args.arg_uri);
        let _ = url.set_port(Some(port));
        client.post(url)
            .send()
            .unwrap();
    }
}
