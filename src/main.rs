mod cli;
mod config;
mod container;
mod errors;
mod ipc;
mod child;
mod hostname;
mod mounts;
mod namespaces;

use errors::exit_with_return_code;

#[macro_use]
extern crate scan_fmt;

fn main() {
    let args = cli::parse_args().expect("Failed to parse arguments");

    if args.debug {
        cli::setup_log(log::LevelFilter::Debug)
    } else {
        cli::setup_log(log::LevelFilter::Info)
    }
    log::info!("{:?}", args);

    exit_with_return_code(container::start(args));
}
