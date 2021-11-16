use crate::structure::registry::EntryFilterResult::{Exclude, IncludeAllChildren};
use crate::structure::registry::{DynamicStateType, LinkEntry};
use crate::structure::EntryAssociatedData::*;
use crate::structure::FileBehaviour::*;
use crate::structure::{DirEntry, FileBehaviour, FileEntry, FilesystemStructure};
use ipc::generated::CommandType;
use ipc::BodyType::*;

#[allow(unused_variables)]
pub fn create_structure() -> FilesystemStructure {
    let mut builder = FilesystemStructure::builder();

    let root = builder.root();

    let player_dir = builder.add_static_entry(root, "player", DirEntry::default());
    builder.add_static_entry(
        player_dir,
        "name",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadOnly(CommandType::PlayerName, String))
            .finish(),
    );

    builder.add_static_entry(
        player_dir,
        "entity",
        LinkEntry::build(|state| {
            Some(format!("../entities/by-id/{}", state.player_entity_id?).into())
        })
        .filter(|state| state.is_in_game())
        .finish(),
    );

    let entities_dir = builder.add_static_entry(
        root,
        "entities",
        DirEntry::build()
            .filter(|state| {
                if state.is_in_game() {
                    IncludeAllChildren
                } else {
                    Exclude
                }
            })
            .finish(),
    );
    let entities_by_id_dir = builder.add_static_entry(
        entities_dir,
        "by-id",
        DirEntry::build()
            .dynamic(DynamicStateType::EntityIds, |state, reg| {
                for id in &state.entity_ids {
                    let entity_dir = reg.add_root_entry(id.to_string(), DirEntry::default());
                    reg.add_static_entry(
                        entity_dir,
                        "health",
                        FileEntry::build()
                            .behaviour(ReadWrite(CommandType::EntityHealth, Float))
                            .associated_data(EntityId(*id))
                            .finish(),
                    );
                    reg.add_static_entry(
                        entity_dir,
                        "type",
                        FileEntry::build()
                            .behaviour(ReadOnly(CommandType::EntityType, String))
                            .associated_data(EntityId(*id))
                            .finish(),
                    );
                    reg.add_static_entry(
                        entity_dir,
                        "position",
                        FileEntry::build()
                            .behaviour(ReadWrite(CommandType::EntityPosition, Position))
                            .associated_data(EntityId(*id))
                            .finish(),
                    );
                }
            })
            .finish(),
    );

    builder.finish()
}
