use crate::structure::registry::StructureBuilder;
use registry::{DirEntry, EntryRef, FileEntry};

mod registry;
pub use registry::{Entry, FilesystemStructure};

macro_rules! file_entry {
    ($ty:ty, $inode:expr) => {
        impl $ty {
            pub const INODE: u64 = $inode;
        }

        impl FileEntry for $ty {}
    };
}

macro_rules! dir_entry {
    ($ty:ident, $inode:expr, $children:expr) => {
        impl $ty {
            pub const INODE: u64 = $inode;
        }

        impl DirEntry for $ty {
            fn children(&self) -> &'static [EntryRef] {
                &$children
            }
        }
    };
}

pub struct RootDir;
dir_entry!(
    RootDir,
    1,
    [EntryRef::File(&TestFile), EntryRef::Dir(&TestDir)]
);

pub struct TestFile;
file_entry!(TestFile, 2);

pub struct TestDir;
dir_entry!(TestDir, 3, []);

#[allow(unused_variables)]
fn register_all(builder: &mut StructureBuilder) {
    // TODO register via inventory crate
    // TODO registration macros

    macro_rules! register_dir {
        ($ty:ident, $name:expr) => {
            builder.register($ty::INODE, $name, Entry::dir($ty));
        };
    }

    macro_rules! register_file {
        ($ty:ident, $name:expr) => {
            builder.register($ty::INODE, $name, Entry::file($ty));
        };
    }

    register_dir!(RootDir, "");
    register_file!(TestFile, "myfile");
    register_dir!(TestDir, "mydir");
}
