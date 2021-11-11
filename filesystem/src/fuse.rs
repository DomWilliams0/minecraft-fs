use crate::structure::{Entry, FilesystemStructure};
use fuser::{FileAttr, FileType, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use log::*;
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};

pub struct MinecraftFs {
    uid: u32,
    gid: u32,
    structure: FilesystemStructure,
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

impl MinecraftFs {
    pub fn new() -> Self {
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
        }
    }

    fn mk_attr(&self, ino: u64, entry: &Entry) -> FileAttr {
        let time = SystemTime::now();
        let kind = match entry {
            Entry::File(_) => FileType::RegularFile,
            Entry::Dir(_) => FileType::Directory,
        };
        FileAttr {
            ino,
            size: 0,
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
