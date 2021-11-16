mod inode;
mod registry;
#[allow(clippy::module_inception)]
mod structure;

pub use registry::{
    DirEntry, Entry, EntryAssociatedData, EntryFilterResult, FileBehaviour, FileEntry,
    FilesystemStructure,
};
pub use structure::create_structure;
