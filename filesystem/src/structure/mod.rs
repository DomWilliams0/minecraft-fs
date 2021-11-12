mod registry;
pub use registry::{Entry, EntryFilterResult, FilesystemStructure};

mod structure {
    #![allow(clippy::module_inception)]

    use super::*;
    use crate::state::GameState;
    use ipc::{generated::CommandType, ReadCommand, ResponseType};
    use registry::{DirEntry, EntryRef, FileEntry, Registration};

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

    dir_entry!(
        PlayerDir,
        "player",
        [
            EntryRef::File(&PlayerHealth),
            EntryRef::File(&PlayerName),
            EntryRef::File(&PlayerPosition)
        ]
    );

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

    dir_entry!(WorldDir, "world", []);
    // file_entry!(WorldName, "name");
}
