mod inode;
mod registry;

pub use inode::InodePool;
pub use registry::{Entry, EntryFilterResult, EntryRef, FilesystemEntry, FilesystemStructure};

mod structure {
    #![allow(clippy::module_inception)]

    use super::*;
    use crate::state::{GameState, GameStateInterest};
    use crate::structure::registry::FilesystemEntry;

    use ipc::generated::CommandArgs;
    use ipc::{generated::CommandType, ReadCommand, ResponseType};
    use registry::{DirEntry, EntryRef, FileEntry, Registration};

    use std::ffi::{OsStr, OsString};

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
            Some(ReadCommand::Stateless(
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
            Some(ReadCommand::Stateless(
                CommandType::PlayerName,
                ResponseType::String,
            ))
        }
    }

    file_entry!(PlayerPosition, "position");
    impl FileEntry for PlayerPosition {
        fn read(&self) -> Option<ReadCommand> {
            Some(ReadCommand::Stateless(
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
            &[EntryRef::Dir(&EntitiesByIdDir)]
        }

        fn filter(&self, state: &GameState) -> EntryFilterResult {
            if !state.is_in_game {
                EntryFilterResult::Exclude
            } else {
                EntryFilterResult::IncludeAllChildren
            }
        }
    }

    dir_entry!(EntitiesByIdDir, "by-id");
    impl DirEntry for EntitiesByIdDir {
        fn children(&self) -> &'static [EntryRef] {
            &[]
        }

        fn dynamic_children(&self, children_out: &mut Vec<FilesystemEntry>, state: &GameState) {
            children_out.extend(state.entity_ids.iter().map(|i| {
                FilesystemEntry::new(OsString::from(i.to_string()), Entry::dir(EntityDir(*i)))
            }));
        }

        fn register_interest(&self, interest: &mut GameStateInterest) {
            interest.entities_by_id = true;
        }
    }

    struct EntityDir(i32);
    impl DirEntry for EntityDir {
        fn children(&self) -> &'static [EntryRef] {
            &[]
        }

        fn dynamic_children(&self, children_out: &mut Vec<FilesystemEntry>, _state: &GameState) {
            children_out.push(FilesystemEntry::new(
                OsStr::new("type"),
                Entry::file(EntityType(self.0)),
            ));

    struct EntityType(i32);
    impl FileEntry for EntityType {
        fn read(&self) -> Option<ReadCommand> {
            Some(ReadCommand::Stateful(
                CommandArgs {
                    cmd: CommandType::EntityType,
                    target_entity: Some(self.0),
                },
                ResponseType::String,
            ))
        }
    }

    entity_file_entry!(EntityType, CommandType::EntityType, ResponseType::String);

    entity_file_entry!(EntityHealth, CommandType::EntityHealth, ResponseType::Float);
}
