use fuser::{ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use std::ffi::OsStr;

pub struct MinecraftFs;

impl fuser::Filesystem for MinecraftFs {
    fn lookup(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        reply.error(libc::EADDRINUSE);
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        reply: ReplyDirectory,
    ) {
        reply.error(libc::EAFNOSUPPORT);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyAttr) {
        reply.error(libc::EACCES);
    }
}
