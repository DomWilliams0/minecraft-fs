use std::error::Error;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::fuse::MinecraftFs;
use fuser::BackgroundSession;
use ipc::IpcChannel;
use parking_lot::{Condvar, Mutex};

struct Mounter(BackgroundSession);
pub struct MountStatus;

static MOUNTER: Mutex<Option<Mounter>> = parking_lot::const_mutex(None);
static CVAR: Condvar = Condvar::new();

pub fn mount(ipc: IpcChannel, path: &Path, opts: &[&str]) -> Result<MountStatus, Box<dyn Error>> {
    ctrlc::set_handler(|| {
        let mut guard = MOUNTER.lock();
        *guard = None;
        CVAR.notify_all();
    })?;

    log::info!("mounting at {}", path.display());
    // TODO this is ridiculous but temporary
    let opts = opts
        .iter()
        .map(|s| OsStr::from_bytes(s.as_bytes()))
        .collect::<Vec<_>>();
    let mnt = fuser::spawn_mount(MinecraftFs::new(ipc), path, &opts)?;
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
