use std::path::PathBuf;
use structopt::StructOpt;

use crate::errors::ErrCode;

#[derive(Debug, StructOpt)]
#[structopt(name = "crabcan", about = "A simple container in Rust.")]
pub struct Args {
    /// Debug mode
    #[structopt(short, long)]
    pub debug: bool,

    /// Command to execute within container
    #[structopt(short, long)]
    pub command: String,

    /// User ID to create inside container
    #[structopt(short, long)]
    pub uid: u32,

    /// Path to mount into the container
    #[structopt(parse(from_os_str), short = "m", long = "mount")]
    pub mount_dir: PathBuf,
}

pub fn parse_args() -> Result<Args, ErrCode> {
    let args = Args::from_args();

    Ok(args)
}

pub fn setup_log(level: log::LevelFilter) {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .filter(None, level)
        .init();
}
