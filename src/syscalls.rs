use crate::errors::ErrCode;
use syscallz::{Action, Cmp, Comparator, Context, Syscall};

use libc::TIOCSTI;
use nix::sched::CloneFlags;
use nix::sys::stat::Mode;

const EPERM: u16 = 1;

fn refuse_syscall(ctx: &mut Context, sc: &Syscall) -> Result<(), ErrCode> {
    match ctx.set_action_for_syscall(Action::Errno(EPERM), *sc) {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrCode::SyscallsError(2)),
    }
}

fn refuse_if_comp(ctx: &mut Context, ind: u32, sc: &Syscall, biteq: u64) -> Result<(), ErrCode> {
    match ctx.set_rule_for_syscall(
        Action::Errno(EPERM),
        *sc,
        &[Comparator::new(ind, Cmp::MaskedEq, biteq, Some(biteq))],
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrCode::SyscallsError(3)),
    }
}

pub fn setsyscalls() -> Result<(), ErrCode> {
    log::debug!("Filtering unwanted syscalls");
    let s_isuid: u64 = Mode::S_ISUID.bits().into();
    let s_isgid: u64 = Mode::S_ISGID.bits().into();
    let clone_new_user: u64 = CloneFlags::CLONE_NEWUSER.bits() as u64;

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
    let syscalls_refuse_ifcomp = [
        (Syscall::chmod, 1, s_isuid),
        (Syscall::chmod, 1, s_isgid),
        (Syscall::fchmod, 1, s_isuid),
        (Syscall::fchmod, 1, s_isgid),
        (Syscall::fchmodat, 2, s_isuid),
        (Syscall::fchmodat, 2, s_isgid),
        (Syscall::unshare, 0, clone_new_user),
        (Syscall::clone, 0, clone_new_user),
        (Syscall::ioctl, 1, TIOCSTI),
    ];

    // Initialize seccomp profile with
    // all syscalls allowed by default
    if let Ok(mut ctx) = Context::init_with_action(Action::Allow) {
        for (sc, ind, biteq) in syscalls_refuse_ifcomp.iter() {
            refuse_if_comp(&mut ctx, *ind, sc, *biteq)?;
        }

        for sc in syscalls_refused.iter() {
            refuse_syscall(&mut ctx, sc)?;
        }

        if let Err(_) = ctx.load() {
            return Err(ErrCode::SyscallsError(0));
        }

        Ok(())
    } else {
        Err(ErrCode::SyscallsError(1))
    }
}
