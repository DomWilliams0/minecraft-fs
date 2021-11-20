use crate::structure::registry::EntryFilterResult::{Exclude, IncludeAllChildren};
use crate::structure::registry::{
    DynamicDirRegistrationer, DynamicStateType, FilesystemStructureBuilder, LinkEntry,
};
use crate::structure::EntryAssociatedData::*;
use crate::structure::FileBehaviour::*;
use crate::structure::{
    DirEntry, EntryAssociatedData, FileBehaviour, FileEntry, FilesystemStructure,
};
use ipc::generated::{CommandType, Dimension};
use ipc::BodyType;
use ipc::BodyType::*;

#[allow(unused_variables)]
pub fn create_structure() -> FilesystemStructure {
    let mut builder = FilesystemStructure::builder();

    player_dir(&mut builder);
    worlds_dir(&mut builder);

    let root = builder.root();
    builder.finish()
}

fn player_dir(builder: &mut FilesystemStructureBuilder) -> u64 {
    let dir = builder.add_entry(builder.root(), "player", DirEntry::default());
    builder.add_entry(
        dir,
        "name",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadOnly(CommandType::PlayerName, String))
            .finish(),
    );

    builder.add_entry(
        dir,
        "entity",
        LinkEntry::build(|state| {
            Some(format!("world/entities/by-id/{}", state.player_entity_id?).into())
        })
        .filter(|state| state.is_in_game())
        .finish(),
    );

    builder.add_entry(
        dir,
        "world",
        LinkEntry::build(|state| {
            let dim = state.player_world.and_then(|dim| match dim {
                Dimension::Overworld => Some("overworld"),
                Dimension::Nether => Some("nether"),
                Dimension::End => Some("end"),
                _ => None,
            })?;
            Some(format!("../worlds/{}", dim).into())
        })
        .filter(|state| state.is_in_game())
        .finish(),
    );

    let control = builder.add_entry(dir, "control", DirEntry::default());
    builder.add_entry(
        control,
        "say",
        FileEntry::build()
            .behaviour(FileBehaviour::WriteOnly(
                CommandType::ControlSay,
                BodyType::String,
            ))
            .finish(),
    );

    builder.add_entry(
        control,
        "jump",
        FileEntry::build()
            .behaviour(FileBehaviour::WriteOnly(
                CommandType::ControlJump,
                BodyType::String, // TODO no input expected?
            ))
            .finish(),
    );

    builder.add_entry(
        control,
        "move",
        FileEntry::build()
            .behaviour(FileBehaviour::WriteOnly(
                CommandType::ControlMove,
                BodyType::Position,
            ))
            .finish(),
    );

    dir
}

fn worlds_dir(builder: &mut FilesystemStructureBuilder) -> u64 {
    let dir = builder.add_entry(
        builder.root(),
        "worlds",
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

    let worlds = [
        ("overworld", Dimension::Overworld),
        ("nether", Dimension::Nether),
        ("end", Dimension::End),
    ];

    for (name, dimension) in worlds {
        let world = builder.add_entry(
            dir,
            name,
            DirEntry::build()
                .associated_data(EntryAssociatedData::World(dimension))
                .finish(),
        );

        entities_dir(builder, world);

        builder.add_entry(
            world,
            "time",
            FileEntry::build()
                .behaviour(FileBehaviour::ReadWrite(CommandType::WorldTime, Integer))
                .finish(),
        );
    }

    dir
}

fn entities_dir(builder: &mut FilesystemStructureBuilder, root: u64) -> u64 {
    let dir = builder.add_entry(
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

    builder.add_entry(
        dir,
        "by-id",
        DirEntry::build()
            .dynamic(DynamicStateType::EntityIds, |state, reg| {
                for id in &state.entity_ids {
                    let entity_dir = reg.add_root_entry(
                        id.to_string(),
                        DirEntry::build()
                            .associated_data(EntryAssociatedData::EntityId(*id))
                            .finish(),
                    );

                    mk_entity_dir(reg, entity_dir);
                }
            })
            .finish(),
    );
    dir
}

fn mk_entity_dir(reg: &mut DynamicDirRegistrationer, entity_dir: u64) {
    reg.add_entry(
        entity_dir,
        "health",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityHealth, Float))
            .finish(),
    );
    reg.add_entry(
        entity_dir,
        "type",
        FileEntry::build()
            .behaviour(ReadOnly(CommandType::EntityType, String))
            .finish(),
    );
    reg.add_entry(
        entity_dir,
        "position",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityPosition, Position))
            .finish(),
    );
}
