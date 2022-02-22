use std::error::Error;
use std::path::Path;

use fuser::{BackgroundSession, MountOption, Session};
use parking_lot::{Condvar, Mutex};

use ipc::IpcChannel;

use crate::fuse::MinecraftFs;

struct Mounter(BackgroundSession);
pub struct MountStatus;

static MOUNTER: Mutex<Option<Mounter>> = parking_lot::const_mutex(None);
static CVAR: Condvar = Condvar::new();

pub fn mount(ipc: IpcChannel, path: &Path) -> Result<MountStatus, Box<dyn Error>> {
    ctrlc::set_handler(|| {
        let mut guard = MOUNTER.lock();
        *guard = None;
        CVAR.notify_all();
    })?;

    log::debug!("Mounting at {}", path.display());
    let opts = [
        MountOption::FSName("minecraft-fs".to_owned()),
        MountOption::RW,
    ];
    let mnt = Session::new(MinecraftFs::new(ipc), path, &opts).and_then(|se| se.spawn())?;
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
