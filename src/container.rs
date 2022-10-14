use std::os::unix::io::RawFd;

use nix::unistd::{close, Pid};

use crate::child::generate_child_process;
use crate::cli::Args;
use crate::config::ContainerOpts;
use crate::errors::ErrCode;

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
    child_pid: Option<Pid>
}

impl Container {
    pub fn new(args: Args) -> Result<Container, ErrCode> {
        let (config, sockets) = ContainerOpts::new(&args.command, args.uid, args.mount_dir)?;
        Ok(Container { sockets, config, child_pid: None })
    }

    pub fn create(&mut self) -> Result<(), ErrCode> {
        let pid = generate_child_process(self.config.clone())?;
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

        Ok(())
    }
}

pub fn start(args: Args) -> Result<(), ErrCode> {
    check_linux_version()?;
    let mut container = Container::new(args)?;
    if let Err(e) = container.create() {
        container.clean_exit();
        log::error!("Error while creating container: {:?}", e);
        return Err(e);
    }
    log::debug!("Finished, cleaning and exiting");
    container.clean_exit()
}
