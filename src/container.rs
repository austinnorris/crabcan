use std::os::unix::io::RawFd;
use std::path::PathBuf;

use nix::sys::wait::waitpid;
use nix::unistd::{close, Pid};

use crate::child::generate_child_process;
use crate::cli::Args;
use crate::config::ContainerOpts;
use crate::errors::ErrCode;
use crate::mounts::clean_mounts;
use crate::namespaces::handle_child_uid_map;
use crate::resources::{clean_cgroups, restrict_resources};

pub const MINIMAL_KERNEL_VERSION: f32 = 4.8;

pub fn check_linux_version() -> Result<(), ErrCode> {
    let host = nix::sys::utsname::uname();
    log::debug!("Linux release: {}", host.release());

    if let Ok(version) = scan_fmt!(host.release(), "{f}.{}", f32) {
        if version < MINIMAL_KERNEL_VERSION {
            return Err(ErrCode::NotSupported(0));
        }
    } else {
        return Err(ErrCode::ContainerError(0));
    }

    if host.machine() != "x86_64" {
        return Err(ErrCode::NotSupported(1));
    }

    Ok(())
}

pub struct Container {
    sockets: (RawFd, RawFd),
    config: ContainerOpts,
    child_pid: Option<Pid>,
}

impl Container {
    pub fn new(args: Args) -> Result<Container, ErrCode> {
        let mut addpaths = vec![];
        for ap_pair in args.addpaths.iter() {
            let mut pair = ap_pair.to_str().unwrap().split(":");
            let frompath = PathBuf::from(pair.next().unwrap())
                .canonicalize()
                .expect("Cannot canonicalize path")
                .to_path_buf();
            let mountpath = PathBuf::from(pair.next().unwrap())
                .strip_prefix("/")
                .expect("Cannot strip prefix from path")
                .to_path_buf();
            addpaths.push((frompath, mountpath))
        }

        let (config, sockets) = ContainerOpts::new(
            &args.command,
            args.uid,
            args.mount_dir,
            args.hostname,
            addpaths,
        )?;
        Ok(Container {
            sockets,
            config,
            child_pid: None,
        })
    }

    pub fn create(&mut self) -> Result<(), ErrCode> {
        let pid = generate_child_process(self.config.clone())?;
        restrict_resources(&self.config.hostname, pid)?;
        handle_child_uid_map(pid, self.sockets.0)?;
        self.child_pid = Some(pid);
        log::debug!("Creation finished");
        Ok(())
    }

    pub fn clean_exit(&mut self) -> Result<(), ErrCode> {
        log::debug!("Cleaning container");

        if let Err(e) = close(self.sockets.0) {
            log::error!("Unable to close write socket: {:?}", e);
            return Err(ErrCode::SocketError(3));
        }

        if let Err(e) = close(self.sockets.1) {
            log::error!("Unable to close read socket: {:?}", e);
            return Err(ErrCode::SocketError(3));
        }

        clean_mounts(&self.config.mount_dir)?;

        if let Err(e) = clean_cgroups(&self.config.hostname) {
            log::error!("Cleaning cgroups failed: {}", e);
            return Err(e);
        }

        Ok(())
    }
}

pub fn wait_child(pid: Option<Pid>) -> Result<(), ErrCode> {
    if let Some(child_pid) = pid {
        log::debug!("Waiting for child (pid-{}) to finish", child_pid);
        if let Err(e) = waitpid(child_pid, None) {
            log::error!("Error while waiting for child to finish: {:?}", e);
            return Err(ErrCode::ContainerError(1));
        }
    }
    Ok(())
}

pub fn start(args: Args) -> Result<(), ErrCode> {
    check_linux_version()?;
    let mut container = Container::new(args)?;
    log::debug!(
        "Container sockets: ({}, {})",
        container.sockets.0,
        container.sockets.1
    );
    if let Err(e) = container.create() {
        container.clean_exit().expect("Exit failure");
        log::error!("Error while creating container: {:?}", e);
        return Err(e);
    }
    log::debug!("Container child PID: {:?}", container.child_pid);
    wait_child(container.child_pid)?;
    log::debug!("Finished, cleaning and exiting");
    container.clean_exit()
}
