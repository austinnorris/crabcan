use crate::config::ContainerOpts;
use crate::errors::ErrCode;

use nix::sched::clone;
use nix::sched::CloneFlags;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

const STACK_SIZE: usize = 1024 * 1024;

fn child(config: ContainerOpts) -> isize {
    log::info!(
        "Starting container with command {} and args {:?}",
        config
            .path
            .to_str()
            .expect("Could not convert path to string"),
        config.argv
    );
    0
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
