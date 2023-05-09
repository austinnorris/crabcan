use crate::errors::ErrCode;

use capctl::caps::Cap;
use capctl::caps::FullCapState;

const CAPABILITIES_DROP: [Cap; 21] = [
                        // Drop because it...
    Cap::AUDIT_CONTROL, // allows access to audit system of the kernel
    Cap::AUDIT_READ,    // allows access to audit system of the kernel
    Cap::AUDIT_WRITE,   // allows access to audit system of the kernel
    Cap::BLOCK_SUSPEND, // prevents system from suspending (suspend is not namespaced)
    Cap::DAC_READ_SEARCH, // allows access to arbitrary files by guessing inode numbers
    Cap::DAC_OVERRIDE, // allows bypass of file read, write, execute permission checks
    Cap::FSETID, // allows modification of setuid executable without removing setuid bit
    Cap::IPC_LOCK, // allows bypass of soft resource limit when locking memory
    Cap::MAC_ADMIN, // used by AppArmor, SELinux, etc. and not namespaced,
                    // hence could be used to circumvent these security mechanisms
    Cap::MAC_OVERRIDE, // same as above
    Cap::MKNOD, // allows programs to (re)create device files, even existing hardware
    Cap::SETFCAP, // allows setting of capabilities on a file
    Cap::SYSLOG, // allows privileged syslog operations and view kernel 
    Cap::SYS_ADMIN, // allows a ton of stuff
    Cap::SYS_BOOT, // allows rebooting and loading new kernels
    Cap::SYS_MODULE, // allows loading or unloading kernel modules
    Cap::SYS_NICE, // allows setting higher priority on pids than default
    Cap::SYS_RAWIO, // allows raw access to the IO ports
    Cap::SYS_RESOURCE, // allows circumventing kernel-wide limits
    Cap::SYS_TIME, // allows setting the time (and not namespaced)
    Cap::WAKE_ALARM // allows interference with suspend (like CAP_BLOCK_SUSPEND)
];

pub fn setcapabilities() -> Result<(), ErrCode> {
    log::debug!("Dropping unwanted capabilities...");
    if let Ok(mut caps) = FullCapState::get_current() {
        caps.bounding.drop_all(CAPABILITIES_DROP.iter().map(|&cap| cap));
        caps.inheritable.drop_all(CAPABILITIES_DROP.iter().map(|&cap| cap));
        Ok(())
    } else {
        Err(ErrCode::CapabilitiesError(0))
    }
}