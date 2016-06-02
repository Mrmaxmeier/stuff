extern crate toml;
extern crate docopt;
extern crate rustc_serialize;
extern crate walkdir;
extern crate lazysort;



use std::process;
use std::collections::HashSet;
use std::io::prelude::*;
use std::fs::File;
use lazysort::SortedBy;
use std::process::Command;


static USAGE: &'static str = r"
Usage:
    cargo
    cargo rebuild-binaries
    cargo rebuild [--all|--outdated]
    cargo [--all|--outdated]
Options:
    --all                   Rebuild all binaries.
    --outdated              Rebuild if older then rustc.
    -h --help               Show this help page.
    --version               Show version.
Rebuild binaries installed with cargo.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    flag_all: bool,
    flag_outdated: bool,
    flag_version: bool,
}

fn get_binaries(outdated: bool,
                cargo_dir: &std::path::PathBuf)
                -> Result<HashSet<String>, std::io::Error> {
    let mut path = cargo_dir.clone();
    path.push("bin");
    println!("{:?}", path);
    let bin_iter = try!(path.read_dir()).filter_map(|res| res.ok());
    let filtered_iter: Vec<_> = if outdated {
        let mut rustc_path = path.clone();
        rustc_path.push("rustc");
        unimplemented!();
        // let rustc_metadata = try!(path.metadata());
        // bin_iter.filter(|bin| true).collect()
    } else {
        bin_iter.collect()
    };

    let binaries = filtered_iter.iter()
        .map(|b| b.file_name().into_string().unwrap())
        .filter(|b| match b.as_ref() {
            "rustc" | "rustdoc" | "cargo" | "multirust" | "rustup" => false,
            _ => true,
        })
        .collect();

    Ok(binaries)
}

fn read_manifest(data: String) -> Option<(String, Vec<String>)> {
    let manifest = match toml::Parser::new(&data).parse() {
        Some(val) => val,
        None => return None,
    };
    let package = match manifest.get("package") {
        Some(&toml::Value::Table(ref package)) => package,
        _ => return None,
    };

    let name = match package.get("name") {
        Some(&toml::Value::String(ref name)) => name.to_owned(),
        _ => return None,
    };

    let bin_tables = match manifest.get("bin") {
        Some(&toml::Value::Array(ref bin_tables)) => bin_tables,
        _ => return Some((name.clone(), vec![name])),
    };

    let binaries = bin_tables.iter()
        .filter_map(|bin| {
            let tab = match *bin {
                toml::Value::Table(ref tab) => tab,
                _ => return None,
            };
            match tab.get("name") {
                Some(&toml::Value::String(ref bin_name)) => Some(bin_name),
                _ => None,
            }
        })
        .cloned()
        .collect();

    Some((name, binaries))
}


fn main() {
    let args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode::<Args>())
        .unwrap_or_else(|err| err.exit());

    if args.flag_version {
        println!("cargo-rebuild-binaries version {}",
                 env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let mut cargo_dir = std::env::home_dir().unwrap();
    cargo_dir.push(".cargo");

    let outdated = args.flag_outdated && !args.flag_all;
    let binaries = get_binaries(outdated, &cargo_dir).unwrap();

    println!("bins: {:?}", binaries);

    let mut registry_path = cargo_dir.clone();
    registry_path.push("registry");
    registry_path.push("src");

    let wk = walkdir::WalkDir::new(registry_path).min_depth(3).max_depth(3);
    let packages = wk.into_iter()
        .filter_map(|res| res.ok())
        .sorted_by(|resa, resb| {
            let meta = resa.metadata().unwrap();
            let metb = resb.metadata().unwrap();
            let a = meta.modified().unwrap();
            let b = metb.modified().unwrap();
            b.cmp(&a)
        })
        .filter(|file| file.file_name() == "Cargo.toml")
        .filter_map(|file| {
            // println!("{:?}", file);
            let mut data = String::new();
            File::open(file.path()).unwrap().read_to_string(&mut data).unwrap();
            read_manifest(data)
        })
        .flat_map(|(package, binaries)| -> Vec<(String, String)> {
            binaries.iter()
                .map(|b| (package.to_owned(), (*b).to_owned()))
                .collect()
        })
        .filter(|&(_, ref bin)| binaries.contains(bin));

    let mut processed_binaries = HashSet::new();
    for (package, binary) in packages {
        if processed_binaries.contains(&binary) {
            continue;
        }
        println!("rebuilding {} [{}]", binary, package);
        processed_binaries.insert(binary);

        let mut cmd = Command::new("cargo");
        cmd.arg("install")
            .arg("--force")
            .arg(package);

        println!("$ {:?}", cmd);

        cmd.spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
