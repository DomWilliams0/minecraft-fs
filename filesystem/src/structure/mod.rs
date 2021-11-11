mod registry;
pub use registry::{Entry, FilesystemStructure};

mod structure {
    #![allow(clippy::module_inception)]

    use super::*;
    use registry::{DirEntry, EntryRef, FileEntry, Registration};

    macro_rules! file_entry {
        ($ty:ident, $name:expr) => {
            struct $ty;
            impl $ty {
                fn entry() -> Entry {
                    Entry::file(Self)
                }
            }

            impl FileEntry for $ty {}

            inventory::submit! { Registration {
                name: $name,
                children: &[],
                entry_fn: $ty::entry,
            }}
        };
    }

    macro_rules! dir_entry {
        ($vis:vis $ty:ident, $name:expr, $children:expr) => {
            $vis struct $ty;

            impl $ty {
                fn entry() -> Entry { Entry::dir(Self) }
            }

            impl DirEntry for $ty {
                fn children(&self) -> &'static [EntryRef] {
                    &$children
                }
            }

            inventory::submit! { Registration {
                name: $name,
                children: &$children,
                entry_fn: $ty::entry,
            }}
        };
    }

    dir_entry!(
        pub Root,
        "",
        [EntryRef::Dir(&PlayerDir), EntryRef::Dir(&WorldDir)]
    );

    dir_entry!(PlayerDir, "player", [EntryRef::File(&PlayerHealth)]);
    file_entry!(PlayerHealth, "health");

    dir_entry!(WorldDir, "world", [EntryRef::File(&WorldName)]);
    file_entry!(WorldName, "name");
}
