use crate::errors::ErrCode;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::unistd::{chdir, pivot_root};
use rand::Rng;
use std::fs::remove_dir;
use std::{fs::create_dir_all, path::PathBuf};

pub fn set_mountpoint(mount_dir: &PathBuf, addpaths: &Vec<(PathBuf, PathBuf)>) -> Result<(), ErrCode> {
    log::debug!("Setting mount point");

    // First we (privately) mount / within the container...

    // MS_REC: used in conjunction with MS_BIND, this will perform a recursive bind mount,
    //         so that all (bindable) submounts in the source subtree "are also bind mounted
    //         at the corresponding location in the target subtree"
    // MS_PRIVATE: "mount/unmount events will not propagate into or out of this mount"
    //             (see "shared subtrees")
    mount_directory(
        None,
        &PathBuf::from("/"),
        vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE],
    )?;

    let new_root = PathBuf::from(format!("/tmp/crabcan.{}", random_string(12)));
    log::debug!(
        "Mounting temp directory {}",
        new_root.as_path().to_str().unwrap()
    );

    create_directory(&new_root)?;

    // ...next we bind mount mount_dir to /tmp/crabcan.<random>...
    mount_directory(
        Some(&mount_dir),
        &new_root,
        vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE],
    )?;

    log::debug!("Mounting additional paths...");
    for (inpath, mountpath) in addpaths.iter() {
        let outpath = new_root.join(mountpath);
        create_directory(&outpath)?;
        mount_directory(
            Some(inpath),
            &outpath,
            vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND, MsFlags::MS_RDONLY]
        )?;
    }

    // ...finally we will do a root pivot
    // See: https://man7.org/linux/man-pages/man2/pivot_root.2.html

    // Initially we'll have / and /tmp/crabcan.<random> mounted in the container.
    // Then we set /tmp/crabcan.<random> as the new / in the container while
    // moving the old / to /tmp/crabcan.<random>/oldroot.<random>

    log::debug!("Pivoting root");

    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));

    create_directory(&put_old)?;
    if let Err(_) = pivot_root(&new_root, &put_old) {
        return Err(ErrCode::MountError(4));
    }

    // Lastly, to achieve isolation from host system, must unmount old root
    log::debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));

    if let Err(_) = chdir(&PathBuf::from("/")) {
        return Err(ErrCode::MountError(5));
    }
    unmount_path(&old_root)?;
    delete_dir(&old_root)?;

    Ok(())
}

pub fn clean_mounts(_root: &PathBuf) -> Result<(), ErrCode> {
    // unmount_path(_root)?;
    Ok(())
}

pub fn mount_directory(
    path: Option<&PathBuf>,
    mount_point: &PathBuf,
    flags: Vec<MsFlags>,
) -> Result<(), ErrCode> {
    let mut ms_flags = MsFlags::empty();

    for f in flags.iter() {
        ms_flags.insert(*f);
    }

    match path {
        Some(p) => log::debug!("Mount {:?} -> {:?}", p, mount_point),
        None => ()
    }

    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(p) = path {
                log::error!(
                    "Cannot mount {} to {}: {}",
                    p.to_str().unwrap(),
                    mount_point.to_str().unwrap(),
                    e
                );
            } else {
                log::error!("Cannot remount {}: {}", mount_point.to_str().unwrap(), e);
            }
            Err(ErrCode::MountError(3))
        }
    }
}

pub fn random_string(n: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::thread_rng();

    let name: String = (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    name
}

pub fn create_directory(path: &PathBuf) -> Result<(), ErrCode> {
    match create_dir_all(path) {
        Err(e) => {
            log::error!("Cannot create directory {}: {}", path.to_str().unwrap(), e);
            Err(ErrCode::MountError(2))
        }
        Ok(_) => Ok(()),
    }
}

pub fn unmount_path(path: &PathBuf) -> Result<(), ErrCode> {
    match umount2(path, MntFlags::MNT_DETACH) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Unable to unmount {}: {}", path.to_str().unwrap(), e);
            Err(ErrCode::MountError(0))
        }
    }
}

pub fn delete_dir(path: &PathBuf) -> Result<(), ErrCode> {
    match remove_dir(path.as_path()) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!(
                "Unable to delete directory {}: {}",
                path.to_str().unwrap(),
                e
            );
            Err(ErrCode::MountError(1))
        }
    }
}
