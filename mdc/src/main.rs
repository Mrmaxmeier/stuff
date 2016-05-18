#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;

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
  --port        Server port.
");
// [default: 7227]

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    println!("{}", args.flag_verbose);
    println!("{}", args.arg_uri);
}
