use crate::errors::ErrCode;
use crate::hostname::generate_hostname;
use crate::ipc::generate_socket_pair;

use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ContainerOpts {
    pub path: CString,
    pub argv: Vec<CString>,
    pub uid: u32,
    pub mount_dir: PathBuf,
    pub fd: RawFd,
    pub hostname: String,
    pub addpaths: Vec<(PathBuf, PathBuf)>,
}

impl ContainerOpts {
    pub fn new(
        command: &str,
        uid: u32,
        mount_dir: PathBuf,
        hostname: Option<String>,
        addpaths: Vec<(PathBuf, PathBuf)>,
    ) -> Result<(ContainerOpts, (RawFd, RawFd)), ErrCode> {
        let sockets = generate_socket_pair()?;
        let argv: Vec<CString> = command
            .split_ascii_whitespace()
            .map(|s| CString::new(s).expect("Cannot read arg"))
            .collect();
        let path = argv[0].clone();
        Ok((
            ContainerOpts {
                path,
                argv,
                uid,
                mount_dir,
                fd: sockets.1.clone(),
                hostname: hostname.unwrap_or(generate_hostname()?),
                addpaths,
            },
            sockets,
        ))
    }
}
