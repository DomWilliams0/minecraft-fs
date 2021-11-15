mod inode;
mod registry;

pub use inode::InodePool;
pub use registry::{Entry, EntryFilterResult, EntryRef, FilesystemEntry, FilesystemStructure};

mod structure {
    #![allow(clippy::module_inception)]

    use super::*;
    use crate::state::{GameState, GameStateInterest};
    use crate::structure::registry::{FilesystemEntry, LinkEntry};
    use std::borrow::Cow;

    use ipc::{generated::CommandType, BodyType, Command, CommandState};
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
    macro_rules! entity_file_entry {
        ($ty:ident, $cmd:expr, $resp:expr) => {
            entity_file_entry!(manual $ty, $cmd, $resp, fn write(&self) -> Option<Command> {None} );
        };

        ($ty:ident, $cmd:expr, $resp:expr; write) => {
            entity_file_entry!(
                manual
                $ty,
                $cmd,
                $resp,
                fn write(&self) -> Option<Command> {Some(Command::stateful(
                    $cmd,
                    $resp,
                    CommandState {
                        target_entity: Some(self.0)
                    },
                ))}
            );
        };

        (manual $ty:ident, $cmd:expr, $resp:expr, $write:item) => {
            struct $ty(i32);

            impl FileEntry for $ty {
                fn read(&self) -> Option<Command> {
                    Some(Command::stateful(
                        $cmd,
                        $resp,
                        CommandState {
                            target_entity: Some(self.0),
                        },
                    ))
                }

                    $write
            }
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
                EntryRef::File(&PlayerName),
                EntryRef::Link(&PlayerEntityLink),
            ]
        }
    }

    struct PlayerEntityLink;
    impl PlayerEntityLink {
        fn entry() -> Entry {
            Entry::link(Self)
        }
    }
    inventory::submit! { Registration {
        name : "entity" ,
        entry_fn : PlayerEntityLink :: entry ,
    } }

    impl LinkEntry for PlayerEntityLink {
        fn target(&self, state: &GameState) -> Option<Cow<'static, OsStr>> {
            let id = state.player_entity_id?.to_string();
            let mut path = OsString::from("../entities/by-id/");
            path.push(id);
            Some(path.into())
        }

        fn should_include(&self, state: &GameState) -> bool {
            state.is_in_game()
        }
    }

    file_entry!(PlayerName, "name");
    impl FileEntry for PlayerName {
        fn read(&self) -> Option<Command> {
            Some(Command::stateless(
                CommandType::PlayerName,
                BodyType::String,
            ))
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
            if !state.is_in_game() {
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

            children_out.push(FilesystemEntry::new(
                OsStr::new("health"),
                Entry::file(EntityHealth(self.0)),
            ));

            children_out.push(FilesystemEntry::new(
                OsStr::new("position"),
                Entry::file(EntityPosition(self.0)),
            ));
        }
    }

    entity_file_entry!(EntityType, CommandType::EntityType, BodyType::String);
    entity_file_entry!(EntityHealth, CommandType::EntityHealth, BodyType::Float; write);
    entity_file_entry!(
        EntityPosition,
        CommandType::EntityPosition,
        BodyType::Position; write
    );
}
