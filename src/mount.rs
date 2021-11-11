use std::error::Error;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use fuser::BackgroundSession;
use parking_lot::{Condvar, Mutex};

use crate::filesystem::MinecraftFs;

struct Mounter(BackgroundSession);
pub struct MountStatus;

static MOUNTER: Mutex<Option<Mounter>> = parking_lot::const_mutex(None);
static CVAR: Condvar = Condvar::new();

pub fn mount(path: &Path, opts: &[&str]) -> Result<MountStatus, Box<dyn Error>> {
    ctrlc::set_handler(|| {
        let mut guard = MOUNTER.lock();
        println!("unmounting");
        *guard = None;
        CVAR.notify_all();
    })?;

    println!("mounting");
    // TODO this is ridiculous but temporary
    let opts = opts
        .iter()
        .map(|s| OsStr::from_bytes(s.as_bytes()))
        .collect::<Vec<_>>();
    let mnt = fuser::spawn_mount(MinecraftFs, path, &opts)?;
    {
        let mut guard = MOUNTER.lock();
        *guard = Some(Mounter(mnt));
    }

    Ok(MountStatus)
}

impl MountStatus {
    pub fn wait_for_unmount(self) {
        let mut guard = MOUNTER.lock();
        CVAR.wait(&mut guard);
    }
}
