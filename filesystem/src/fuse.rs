use std::ffi::OsStr;
use std::fmt::Write;
use std::os::unix::ffi::OsStrExt;

use std::time::{Duration, SystemTime};

use fuser::{
    FileAttr, FileType, ReplyAttr, ReplyBmap, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyDirectoryPlus, ReplyEmpty, ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek, ReplyOpen,
    ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
};
use log::*;

use ipc::{IpcChannel, IpcError};

use crate::state::{CachedGameState, GameStateInterest};
use crate::structure::{
    create_structure, Entry, EntryFilterResult, FileBehaviour, FilesystemStructure,
};

pub struct MinecraftFs {
    uid: u32,
    gid: u32,
    ipc: IpcChannel,
    state: CachedGameState,
    structure: FilesystemStructure,
}

// TODO this might be able to be much longer
const TTL: Duration = Duration::from_secs(1);

impl fuser::Filesystem for MinecraftFs {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        trace!("lookup(parent={}, name={:?})", parent, name);
        let (inode, entry) = match self.structure.lookup_child(parent, name) {
            Some(tup) => tup,
            None => {
                let interest = self.structure.interest_for_inode(parent);
                let state = match self.state.get(&mut self.ipc, interest.as_interest()) {
                    Ok(state) => state,
                    Err(err) => {
                        log::error!("failed to fetch game state: {}", err);
                        return reply.error(libc::EIO);
                    }
                };

                self.structure.ensure_generated(state, interest);

                // try again now that dynamic children have been generated
                match self.structure.lookup_child(parent, name) {
                    Some(tup) => tup,
                    None => return reply.error(libc::ENOENT),
                }
            }
        };

        let attr = self.mk_attr(inode, entry);
        reply.entry(&TTL, &attr, 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        trace!("getattr({})", ino);
        let attr = match self.structure.lookup_inode(ino) {
            Some(entry) => self.mk_attr(ino, entry),

            None => return reply.error(libc::ENOENT),
        };

        reply.attr(&TTL, &attr);
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        trace!("setattr(inode={})", ino);

        let entry = match self.structure.lookup_inode(ino) {
            Some(entry) => entry,
            None => return reply.error(libc::ENOENT),
        };

        if let (Some(0), Entry::File(_)) = (size, entry) {
            trace!("truncating file");
            return reply.attr(&TTL, &self.mk_attr(ino, entry));
        }

        reply.error(libc::ENOSYS)
    }

    fn readlink(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyData) {
        trace!("readlink({})", ino);

        match self.structure.lookup_inode(ino) {
            Some(Entry::Link(link)) => {
                let interest = GameStateInterest::default();
                // TODO need to customise state interest? or pass None
                let state = match self.state.get(&mut self.ipc, interest) {
                    Ok(state) => state,
                    Err(err) => {
                        log::error!("failed to fetch game state: {}", err);
                        return reply.error(libc::EIO);
                    }
                };

                match (link.target())(state) {
                    Some(path) => reply.data(path.as_bytes()),
                    None => reply.error(libc::EINVAL),
                }
            }
            _ => reply.error(libc::ENOENT),
        }
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
        // TODO actually check readability in fh? requires implementing open()
        trace!(
            "read(inode={}, fh={}, offset={}, size={})",
            ino,
            fh,
            offset,
            size
        );

        let file = match self.structure.lookup_inode(ino) {
            Some(Entry::File(f)) => f,
            _ => return reply.error(libc::ENOENT),
        };

        let (cmd, resp) = match file.behaviour() {
            Some(FileBehaviour::ReadOnly(cmd, resp) | FileBehaviour::ReadWrite(cmd, resp)) => {
                (cmd, resp)
            }
            _ => return reply.error(libc::EOPNOTSUPP),
        };

        let state = file.command_state();
        let resp = match self.ipc.send_read_command(cmd, resp, state) {
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

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        trace!(
            "write(inode={}, offset={}, data=<{} bytes>)",
            ino,
            offset,
            data.len()
        );

        let file = match self.structure.lookup_inode(ino) {
            Some(Entry::File(f)) => f,
            _ => return reply.error(libc::ENOENT),
        };

        let (cmd, body_type) = match file.behaviour() {
            Some(FileBehaviour::WriteOnly(cmd, body) | FileBehaviour::ReadWrite(cmd, body)) => {
                (cmd, body)
            }
            _ => return reply.error(libc::EOPNOTSUPP),
        };

        let state = file.command_state();
        match self.ipc.send_write_command(cmd, body_type, data, state) {
            Ok(resp) => reply.written(resp as u32),
            Err(err) => {
                error!("write failed: {}", err);
                reply.error(ipc_error_code(&err));
            }
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

        let _dir = match self.structure.lookup_inode(ino) {
            Some(Entry::Dir(dir)) => dir,
            _ => return reply.error(libc::ENOENT),
        };

        let interest = self.structure.interest_for_inode(ino);
        let state = match self.state.get(&mut self.ipc, interest.as_interest()) {
            Ok(state) => state,
            Err(err) => {
                log::error!("failed to fetch game state: {}", err);
                return reply.error(libc::EIO);
            }
        };

        self.structure.ensure_generated(state, interest);

        let all_children = match self.structure.lookup_children(ino) {
            Some(children) => children,
            _ => return reply.error(libc::ENOENT),
        };

        let offset = offset as usize;
        let mut last_filter = None;
        for (i, (child, name)) in all_children.skip(offset).enumerate() {
            if let Some(EntryFilterResult::IncludeAllChildren) = last_filter {
                // dont bother filtering
            } else {
                let filtered = child.filter(state);
                let skip = matches!(filtered, EntryFilterResult::Exclude);
                last_filter = Some(filtered);
                if skip {
                    continue;
                }
            }

            let kind = match child {
                Entry::File(_) => FileType::RegularFile,
                Entry::Dir(_) => FileType::Directory,
                Entry::Link(_) => FileType::Symlink,
            };

            if reply.add(ino, (offset + i + 1) as i64, kind, OsStr::new(name)) {
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
        IpcError::Connecting(_)
        | IpcError::Sending(_)
        | IpcError::Receiving(_)
        | IpcError::Deserialization(_) => libc::EIO,
        IpcError::UnexpectedGameResponse(_)
        | IpcError::UnexpectedResponse(_)
        | IpcError::BadData(_) => libc::EINVAL,
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

        let structure = create_structure();

        Self {
            uid,
            gid,
            ipc,
            state: CachedGameState::default(),
            structure,
        }
    }

    fn mk_attr(&self, ino: u64, entry: &Entry) -> FileAttr {
        let time = SystemTime::now();
        // TODO set file size properly
        let (kind, size) = match entry {
            Entry::File(_) => (FileType::RegularFile, 256),
            Entry::Dir(_) => (FileType::Directory, 0),
            Entry::Link(_) => (FileType::Symlink, 0),
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
