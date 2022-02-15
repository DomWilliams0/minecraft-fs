use ipc::generated::{CommandType, Dimension};
use ipc::BodyType;
use ipc::BodyType::*;

use crate::structure::registry::EntryFilterResult::{Exclude, IncludeAllChildren};
use crate::structure::registry::{
    DynamicDirRegistrationer, DynamicStateType, FilesystemStructureBuilder, LinkEntry,
    PhantomChildType,
};

use crate::structure::FileBehaviour::*;
use crate::structure::{
    DirEntry, EntryAssociatedData, FileBehaviour, FileEntry, FilesystemStructure,
};

#[allow(unused_variables)]
pub fn create_structure() -> FilesystemStructure {
    let mut builder = FilesystemStructure::builder();

    player_dir(&mut builder);
    worlds_dir(&mut builder);

    let root = builder.root();
    builder.finish()
}

fn player_dir(builder: &mut FilesystemStructureBuilder) -> u64 {
    let dir = builder.add_entry(
        builder.root(),
        "player",
        DirEntry::build()
            .associated_data(EntryAssociatedData::PlayerId)
            .dynamic(DynamicStateType::PlayerId, |_, reg| {
                mk_entity_dir(reg, reg.parent(), false);
            })
            .finish(),
    );
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

        let blocks_dir = builder.add_entry(world, "blocks", DirEntry::default());

        builder.add_entry(
            blocks_dir,
            "README",
            FileEntry::build()
                .behaviour(FileBehaviour::Static(
                    "Path format is ./x,y,z\ne.g. 0,64,100\n",
                ))
                .finish(),
        );

        let parse_pos = |name: &str| {
            parse_block_position(name).map(|[x, y, z]| PhantomChildType::Block([x, y, z]))
        };

        builder.add_phantom(
            blocks_dir,
            parse_pos,
            |ty| {
                let PhantomChildType::Block(pos) = ty;
                DynamicStateType::Block(pos)
            },
            |state, reg| {
                // TODO support writing
                let block = state.block.as_ref().expect("missing block details");

                reg.add_root_entry(
                    "type",
                    FileEntry::build()
                        .behaviour(FileBehaviour::ReadOnly(
                            CommandType::BlockType,
                            BodyType::String,
                        ))
                        .finish(),
                );

                // TODO add static file with block pos

                // TODO useful block info
                if block.has_color {
                    reg.add_root_entry(
                        "color",
                        FileEntry::build()
                            .behaviour(FileBehaviour::ReadOnly(
                                CommandType::BlockColor,
                                BodyType::String,
                            ))
                            .finish(),
                    );
                }
            },
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

                    mk_entity_dir(reg, entity_dir, true);
                }
            })
            .finish(),
    );
    dir
}

fn mk_entity_dir(reg: &mut DynamicDirRegistrationer, entity_dir: u64, generic: bool) {
    reg.add_entry(
        entity_dir,
        "health",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityHealth, Float))
            .finish(),
    );
    reg.add_entry(
        entity_dir,
        "position",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityPosition, Position))
            .finish(),
    );

    if generic {
        reg.add_entry(
            entity_dir,
            "type",
            FileEntry::build()
                .behaviour(ReadOnly(CommandType::EntityType, String))
                .finish(),
        );
    }
}

// ------
fn parse_block_position(s: &str) -> Option<[i32; 3]> {
    let mut parts = s.splitn(3, ',').filter_map(|s| s.parse::<i32>().ok());

    match (parts.next(), parts.next(), parts.next()) {
        (Some(x), Some(y), Some(z)) => Some([x, y, z]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_pos_parsing() {
        assert!(matches!(
            parse_block_position("1,400,-64"),
            Some([1, 400, -64])
        ));
        assert!(matches!(parse_block_position("0,0,0"), Some([0, 0, 0])));
        assert!(parse_block_position("oof").is_none());
        assert!(parse_block_position("1,2,3,4").is_none());
        assert!(parse_block_position("500,200").is_none());
        assert!(parse_block_position("123,nice,200").is_none());
    }
}
