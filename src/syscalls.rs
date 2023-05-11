use crate::errors::ErrCode;
use syscallz::{Context, Action, Syscall};

const EPERM: u16 = 1;

fn refuse_syscall(ctx: &mut Context, sc: &Syscall) -> Result<(), ErrCode> {
    match ctx.set_action_for_syscall(Action::Errno(EPERM), *sc) {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrCode::SyscallsError(2))
    }
}

pub fn setsyscalls() -> Result<(), ErrCode> {
    log::debug!("Filtering unwanted syscalls");

    // Unconditional syscall deny
    let syscalls_refused = [
        Syscall::keyctl,
        Syscall::add_key,
        Syscall::request_key,
        Syscall::mbind,
        Syscall::migrate_pages,
        Syscall::move_pages,
        Syscall::set_mempolicy,
        Syscall::userfaultfd,
        Syscall::perf_event_open,
    ];

    // Conditional syscall deny

    // Initialize seccomp profile with
    // all syscalls allowed by default
    if let Ok(mut ctx) = Context::init_with_action(Action::Allow) {

        // Configure profile here

        if let Err(_) = ctx.load() {
            return Err(ErrCode::SyscallsError(0));
        }

        for sc in syscalls_refused.iter() {
            refuse_syscall(&mut ctx, sc)?;
        }

        Ok(())
    } else {
        Err(ErrCode::SyscallsError(1))
    }

}