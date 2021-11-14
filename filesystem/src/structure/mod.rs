mod inode;
mod registry;

pub use inode::InodePool;
pub use registry::{Entry, EntryFilterResult, EntryRef, FilesystemEntry, FilesystemStructure};

mod structure {
    #![allow(clippy::module_inception)]

    use super::*;
    use crate::state::GameState;
    use crate::structure::registry::FilesystemEntry;

    use ipc::{generated::CommandType, ReadCommand, ResponseType};
    use registry::{DirEntry, EntryRef, FileEntry, Registration};
    use std::ffi::OsString;

    macro_rules! file_entry {
        ($ty:ident, $name:expr) => {
            struct $ty;
            impl $ty {
                fn entry() -> Entry {
                    Entry::file(Self)
                }
            }

            inventory::submit! { Registration {
                name: $name,
                entry_fn: $ty::entry,
            }}
        };
    }

    macro_rules! dir_entry {
        ($vis:vis $ty:ident, $name:expr) => {
            $vis struct $ty;

            impl $ty {
                fn entry() -> Entry { Entry::dir(Self) }
            }

            inventory::submit! { Registration {
                name: $name,
                entry_fn: $ty::entry,
            }}
        };
    }

    dir_entry!(
        pub Root,
        ""
    );

    impl DirEntry for Root {
        fn children(&self) -> &'static [EntryRef] {
            &[EntryRef::Dir(&PlayerDir), EntryRef::Dir(&EntitiesDir)]
        }
    }

    dir_entry!(PlayerDir, "player");

    impl DirEntry for PlayerDir {
        fn children(&self) -> &'static [EntryRef] {
            &[
                EntryRef::File(&PlayerHealth),
                EntryRef::File(&PlayerName),
                EntryRef::File(&PlayerPosition),
            ]
        }
    }

    file_entry!(PlayerHealth, "health");
    impl FileEntry for PlayerHealth {
        fn read(&self) -> Option<ReadCommand> {
            Some(ReadCommand::WithResponse(
                CommandType::PlayerHealth,
                ResponseType::Float,
            ))
        }

        fn should_include(&self, state: &GameState) -> bool {
            state.is_in_game
        }
    }

    file_entry!(PlayerName, "name");
    impl FileEntry for PlayerName {
        fn read(&self) -> Option<ReadCommand> {
            Some(ReadCommand::WithResponse(
                CommandType::PlayerName,
                ResponseType::String,
            ))
        }
    }

    file_entry!(PlayerPosition, "position");
    impl FileEntry for PlayerPosition {
        fn read(&self) -> Option<ReadCommand> {
            Some(ReadCommand::WithResponse(
                CommandType::PlayerPosition,
                ResponseType::Position,
            ))
        }

        fn should_include(&self, state: &GameState) -> bool {
            state.is_in_game
        }
    }

    // dir_entry!(WorldDir, "world");
    // file_entry!(WorldName, "name");

    dir_entry!(EntitiesDir, "entities");
    impl DirEntry for EntitiesDir {
        fn children(&self) -> &'static [EntryRef] {
            &[EntryRef::Dir(&EntitiesByTypeDir)]
        }
    }

    dir_entry!(EntitiesByTypeDir, "by-type");
    impl DirEntry for EntitiesByTypeDir {
        fn children(&self) -> &'static [EntryRef] {
            &[]
        }

        fn dynamic_children(&self, children_out: &mut Vec<FilesystemEntry>, state: &GameState) {
            children_out.extend((1..4).map(|i| {
                FilesystemEntry::new(
                    OsString::from(format!("dynamic-boi-{}", i)).into(),
                    Entry::dir(DynamicTest(i)),
                )
            }));
        }
    }

    struct DynamicTest(u32);

    impl DirEntry for DynamicTest {
        fn children(&self) -> &'static [EntryRef] {
            &[EntryRef::File(&MyThing)]
        }
    }

    file_entry!(MyThing, "cool");
    impl FileEntry for MyThing {
        fn read(&self) -> Option<ReadCommand> {
            None
        }
    }
}
