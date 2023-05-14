use crate::config::ContainerOpts;
use crate::errors::ErrCode;
use crate::hostname::set_container_hostname;
use crate::mounts::set_mountpoint;
use crate::namespaces::userns;
use crate::capabilities::setcapabilities;
use crate::syscalls::setsyscalls;

use std::ffi::CString;
use nix::sched::clone;
use nix::sched::CloneFlags;
use nix::sys::signal::Signal;
use nix::unistd::{Pid, close, execve};

const STACK_SIZE: usize = 1024 * 1024;

fn child(config: ContainerOpts) -> isize {
    match setup_container_config(&config) {
        Ok(_) => log::info!("Container configured successfully"),
        Err(e) => {
            log::error!("Error while configuring container: {:?}", e);
            return -1;
        }
    }

    if let Err(_) = close(config.fd) {
        log::error!("Error while closing socket...");
        return -1;
    }

    log::info!(
        "Starting container with command {} and args {:?}",
        config
            .path
            .to_str()
            .expect("Could not convert path to string"),
        config.argv
    );

    let retcode = match execve::<CString, CString>(&config.path, &config.argv, &[]) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Error while performing execve: {:?}", e);
            -1
        }
    };

    retcode

}

pub fn generate_child_process(config: ContainerOpts) -> Result<Pid, ErrCode> {
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS); // start cloned child in new mount namespace
    flags.insert(CloneFlags::CLONE_NEWCGROUP); // used to restrict capabilities of child process
    flags.insert(CloneFlags::CLONE_NEWPID); // child will have own namespace for PIDs
    flags.insert(CloneFlags::CLONE_NEWIPC); // isolate child IPC access to only its own namespace
    flags.insert(CloneFlags::CLONE_NEWNET); // similarly, for network interfaces/configs
    flags.insert(CloneFlags::CLONE_NEWUTS); // isolation of hostname

    match clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    ) {
        Ok(pid) => {
            log::debug!("Child process PID: {}", pid);
            Ok(pid)
        }
        Err(_) => Err(ErrCode::ChildProcessError(0)),
    }
}

fn setup_container_config(config: &ContainerOpts) -> Result<(), ErrCode> {
    set_container_hostname(&config.hostname)?;
    set_mountpoint(&config.mount_dir)?;
    userns(config.fd, config.uid)?;
    setcapabilities()?;
    setsyscalls()?;
    Ok(())
}
