use crate::structure::{Entry, FilesystemStructure};
use fuser::{FileAttr, FileType, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use ipc::{IpcChannel, IpcError};
use log::*;
use std::ffi::OsStr;
use std::fmt::Write;
use std::time::{Duration, SystemTime};

pub struct MinecraftFs {
    uid: u32,
    gid: u32,
    structure: FilesystemStructure,
    ipc: IpcChannel,
}

// TODO this might be able to be much longer
const TTL: Duration = Duration::from_secs(1);

impl fuser::Filesystem for MinecraftFs {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        trace!("lookup(parent={}, name={:?})", parent, name);
        let attr = match self.structure.lookup_child(parent, name) {
            Some((inode, entry)) => self.mk_attr(inode, entry),
            _ => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        reply.entry(&TTL, &attr, 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        trace!("getattr({})", ino);
        let attr = match self.structure.lookup_inode(ino) {
            Some(entry) => self.mk_attr(ino, entry),

            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        reply.attr(&TTL, &attr);
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        // TODO actually check readability in fh, which requires implementing open()
        trace!(
            "read(inode={}, fh={}, offset={}, size={})",
            ino,
            fh,
            offset,
            size
        );

        let file = match self.structure.lookup_inode(ino) {
            Some(Entry::File(f)) => &**f,
            _ => return reply.error(libc::ENOENT),
        };

        let cmd = match file.read() {
            Some(cmd) => cmd,
            None => return reply.error(libc::EOPNOTSUPP),
        };

        let resp = match self.ipc.send_read(cmd) {
            Ok(resp) => resp,
            Err(err) => {
                error!("command failed: {}", err);
                return reply.error(ipc_error_code(&err));
            }
        };

        // TODO respect offset and size
        // TODO reuse allocation
        let mut response_data = String::new();
        match write!(&mut response_data, "{}", resp) {
            Ok(_) => {
                reply.data(response_data.as_bytes());
            }
            Err(_) => reply.error(libc::ENOMEM),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        trace!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);

        let dir = match self.structure.lookup_inode(ino) {
            Some(Entry::Dir(dir)) => &**dir,
            _ => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let offset = offset as usize;
        for (i, child) in dir.children().iter().skip(offset).enumerate() {
            let entry = self.structure.lookup_entry(child);
            if reply.add(ino, (offset + i + 1) as i64, entry.kind(), entry.name()) {
                break;
            }
        }

        reply.ok();
    }
}

fn ipc_error_code(err: &IpcError) -> i32 {
    match err {
        IpcError::NoCurrentGame | IpcError::ClientError(_) => libc::EOPNOTSUPP,
        IpcError::NotFound => libc::ENOENT,
        IpcError::Connecting(_) | IpcError::SendingCommand(_) | IpcError::ReadingResponse(_) => {
            libc::EIO
        }
        IpcError::UnexpectedResponse(_) => libc::EINVAL,
    }
}

impl MinecraftFs {
    pub fn new(ipc: IpcChannel) -> Self {
        let uid;
        let gid;

        unsafe {
            uid = libc::getuid();
            gid = libc::getgid();
        }

        Self {
            uid,
            gid,
            structure: FilesystemStructure::new(),
            ipc,
        }
    }

    fn mk_attr(&self, ino: u64, entry: &Entry) -> FileAttr {
        let time = SystemTime::now();
        // TODO set file size properly
        let (kind, size) = match entry {
            Entry::File(_) => (FileType::RegularFile, 256),
            Entry::Dir(_) => (FileType::Directory, 0),
        };
        FileAttr {
            ino,
            size,
            blocks: 0,
            atime: time,
            mtime: time,
            ctime: time,
            crtime: time,
            kind,
            perm: 0o755,
            nlink: 1,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }
}
