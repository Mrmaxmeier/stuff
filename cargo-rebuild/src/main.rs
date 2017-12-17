#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;
extern crate docopt;
extern crate rustc_serialize;
extern crate walkdir;
extern crate lazysort;
extern crate ansi_term;

use std::process;
use std::collections::HashSet;
use std::io::prelude::*;
use std::fs::File;

use toml::Value;
use lazysort::{SortedBy, Sorted};
use ansi_term::Style;


static USAGE: &'static str = r"
Usage:
    cargo rebuild [options]

Options:
    -a --all                Rebuild all binaries.
    -o --outdated           Rebuild if older then rustc.
    -v --verbose            Enable verbose logging.
    -d --dry-run            Don't actually invoke `cargo install`.
    -h --help               Show this help page.
    -V --version            Show version.

Rebuild binaries installed with cargo.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    flag_all: bool,
    flag_outdated: bool,
    flag_dry_run: bool,
    flag_verbose: bool,
    flag_version: bool,
}

fn get_rustup_toolchain(home: &std::path::PathBuf) -> Result<std::path::PathBuf, std::io::Error> {
    let mut settings_file = home.clone();
    settings_file.push(".rustup/settings.toml");

    let mut buffer = try!(File::open(&settings_file));
    let mut data = String::new();
    try!(buffer.read_to_string(&mut data));

    let settings = data.parse::<Value>()?;
    if let Some(&toml::Value::String(ref default_toolchain)) = settings.get("default_toolchain") {
        let folder = if let Some(&toml::Value::String(ref default_triple)) = settings.get("default_host_triple") {
            format!("{}-{}", default_toolchain, default_triple)
        } else {
            default_toolchain.clone()
        };
        let mut rustc_path = home.clone();
        rustc_path.push(".rustup/toolchains");
        rustc_path.push(folder);
        Ok(rustc_path)
    } else {
        panic!("invalid rustup settings file")
    }
}

fn get_rustc_build_time() -> Result<std::time::SystemTime, std::io::Error> {
    let home = std::env::home_dir().unwrap();

    let mut rustc_path = get_rustup_toolchain(&home).unwrap_or_else(|_| {
        let mut rustc_path = home.clone();
        rustc_path.push(".cargo");
        rustc_path
    });
    rustc_path.push("bin/rustc");

    debug!("rustc_path: {:?}", rustc_path);

    try!(rustc_path.metadata()).modified()
}

fn get_binaries(outdated: bool,
                home_dir: &std::path::PathBuf)
                -> Result<HashSet<String>, std::io::Error> {
    let mut path = home_dir.clone();
    path.push(".cargo/bin");
    debug!("{:?}", path);
    let bin_iter = try!(path.read_dir()).filter_map(|res| res.ok());
    let filtered_iter: Vec<_> = if outdated {
        debug!("checking outdatedness");
        let rustc_build_date = try!(get_rustc_build_time());
        bin_iter.filter(|bin| {
                if let Ok(meta) = bin.metadata() {
                    if let Ok(modified) = meta.modified() {
                        debug!("{:?} {:?} {:?} {}",
                               bin.path(),
                               modified,
                               rustc_build_date,
                               modified < rustc_build_date);
                        return modified < rustc_build_date;
                    }
                };
                false
            })
            .collect()
    } else {
        bin_iter.collect()
    };

    let binaries = filtered_iter.iter()
        .map(|b| b.file_name().into_string().unwrap())
        .filter(|b| match b.as_ref() {
            "rustc" | "rustdoc" | "cargo" | "rustup" => false,
            _ => true,
        })
        .collect();

    Ok(binaries)
}

fn read_manifest(data: String) -> Option<(String, Vec<String>)> {
    let manifest = data.parse::<Value>()?;
    let package = manifest["package"]?;
    let name = manifest["name"]?;
    /*
    let package = match manifest.get("package") {
        Some(&toml::Value::Table(ref package)) => package,
        _ => return None,
    };

    let name = match package.get("name") {
        Some(&toml::Value::String(ref name)) => name.to_owned(),
        _ => return None,
    };
    */

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
    env_logger::init().unwrap();
    let args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode::<Args>())
        .unwrap_or_else(|err| err.exit());

    if args.flag_version {
        println!("cargo-rebuild version {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let home_dir = std::env::home_dir().unwrap();

    let only_outdated = args.flag_outdated || !args.flag_all;
    let binaries = get_binaries(only_outdated, &home_dir.clone()).unwrap();

    if binaries.is_empty() {
        println!("Nothing to do!");
        return;
    }

    debug!("binaries: {:?}", binaries);

    let mut registry_path = home_dir.clone();
    registry_path.push(".cargo/registry/src");

    let wk = walkdir::WalkDir::new(registry_path).min_depth(3).max_depth(3);
    let mut package_set = HashSet::new();
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
            debug!("{:?}", file);
            let mut data = String::new();
            File::open(file.path()).unwrap().read_to_string(&mut data).unwrap();
            read_manifest(data)
        })
        .filter_map(|(p, b)| {
            if package_set.contains(&p) {
                None
            } else {
                package_set.insert(p.clone());
                Some((p, b))
            }
        })
        .flat_map(|(package, binaries)| -> Vec<(String, String)> {
            binaries.iter()
                .map(|b| (package.to_owned(), (*b).to_owned()))
                .collect()
        })
        .filter(|&(_, ref bin)| binaries.contains(bin))
        .collect::<Vec<(String, String)>>();

    if packages.is_empty() {
        println!("Nothing to do!");
        return;
    }

    let mut processed_packages = HashSet::new();
    let mut processed_binaries = HashSet::new();
    for &(ref package, ref binary) in &packages {
        if processed_packages.contains(package) {
            continue;
        }
        processed_packages.insert(package.clone());
        if processed_binaries.contains(binary) {
            continue;
        }
        processed_binaries.insert(binary.clone());

        let mut binaries_in_package = HashSet::new();
        for &(ref p, ref b) in &packages {
            if p == package {
                processed_binaries.insert(b.clone());
                binaries_in_package.insert(b);
            }
        }

        print!("{}", Style::new().bold().paint("> Rebuilding"));
        for bin in binaries_in_package.iter().sorted() {
            print!(" {}", bin);
        }
        if binaries_in_package.len() == 1 && package == binary {
            println!("");
        } else {
            println!(" [{}]", Style::new().underline().paint(package.clone()));
        }

        if args.flag_dry_run {
            continue;
        }

        let mut cmd = process::Command::new("cargo");
        cmd.arg("install")
            .arg("--force")
            .arg(package);

        debug!("$ {:?}", cmd);

        cmd.spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
