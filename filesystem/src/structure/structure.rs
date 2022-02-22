use std::borrow::Cow;

use ipc::generated::{CommandType, Dimension, EntityDetails};
use ipc::BodyType;
use ipc::BodyType::*;

use crate::structure::entry::{DirEntry, EntryAssociatedData, FileEntry, LinkEntry};
use crate::structure::registry::EntryFilterResult::{Exclude, IncludeAllChildren};
use crate::structure::registry::{
    DynamicDirRegistrationer, DynamicStateType, FilesystemStructureBuilder, PhantomChildType,
};
use crate::structure::FileBehaviour::*;
use crate::structure::{EntryFilterResult, FileBehaviour, FilesystemStructure};

#[allow(unused_variables)]
pub fn create_structure() -> FilesystemStructure {
    let mut builder = FilesystemStructure::builder();

    player_dir(&mut builder);
    worlds_dir(&mut builder);

    builder.add_entry(
        builder.root(),
        "version",
        FileEntry::build()
            .behaviour(FileBehaviour::Static(Cow::Borrowed(env!(
                "CARGO_PKG_VERSION"
            ))))
            .finish(),
    );

    builder.add_entry(
        builder.root(),
        "command",
        FileEntry::build()
            .behaviour(FileBehaviour::WriteOnly(CommandType::ServerCommand, String))
            .filter(|state| state.is_in_game())
            .finish(),
    );

    builder.finish()
}

enum EntityType<'a> {
    SpecificallyPlayer,
    Other(&'a EntityDetails),
}

fn player_dir(builder: &mut FilesystemStructureBuilder) -> u64 {
    let dir = builder.add_entry(
        builder.root(),
        "player",
        DirEntry::build()
            .associated_data(EntryAssociatedData::PlayerId)
            .dynamic(DynamicStateType::PlayerId, |state, reg| {
                mk_entity_dir(reg, reg.parent(), EntityType::SpecificallyPlayer);
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
        "gamemode",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadWrite(
                CommandType::PlayerGamemode,
                String,
            ))
            .finish(),
    );

    builder.add_entry(
        dir,
        "hunger",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadWrite(CommandType::PlayerHunger, Integer))
            .finish(),
    );

    builder.add_entry(
        dir,
        "saturation",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadWrite(
                CommandType::PlayerSaturation,
                Float,
            ))
            .finish(),
    );
    builder.add_entry(
        dir,
        "exhaustion",
        FileEntry::build()
            .behaviour(FileBehaviour::ReadWrite(
                CommandType::PlayerExhaustion,
                Float,
            ))
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

    let control = builder.add_entry(
        dir,
        "control",
        DirEntry::build()
            .filter(|state| {
                if state.is_in_game() {
                    EntryFilterResult::IncludeAllChildren
                } else {
                    EntryFilterResult::Exclude
                }
            })
            .finish(),
    );
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
                    "Path format is ./x,y,z or ./x\\ y\\ z\ne.g. 0,64,100 or \"0.5 22.3 41.5555\"\n".into(),
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
                let block = state.block.as_ref().expect("missing block details");
                let pos = block.pos;

                reg.add_root_entry(
                    "type",
                    FileEntry::build()
                        .behaviour(FileBehaviour::ReadWrite(
                            CommandType::BlockType,
                            BodyType::String,
                        ))
                        .finish(),
                );

                reg.add_root_entry(
                    "pos",
                    FileEntry::build()
                        .behaviour(FileBehaviour::Static(
                            format!("{},{},{}", pos.x(), pos.y(), pos.z()).into(),
                        ))
                        .finish(),
                );

                let neighbours_dir = reg.add_root_entry("adjacent", DirEntry::default());
                type PosMut<'a> = &'a mut (i32, i32, i32);
                macro_rules! adjacent {
                    ($name:expr, $pos_fn:expr) => {
                        reg.add_entry(
                            neighbours_dir,
                            $name,
                            LinkEntry::build(move |_| {
                                let mut pos = (pos.x(), pos.y(), pos.z());
                                #[allow(clippy::redundant_closure_call)]
                                ($pos_fn)(&mut pos);
                                Some(format!("../../{},{},{}", pos.0, pos.1, pos.2).into())
                            })
                            .finish(),
                        );
                    };
                }

                adjacent!("west", |pos: PosMut| pos.0 -= 1);
                adjacent!("east", |pos: PosMut| pos.0 += 1);
                adjacent!("below", |pos: PosMut| pos.1 -= 1);
                adjacent!("above", |pos: PosMut| pos.1 += 1);
                adjacent!("north", |pos: PosMut| pos.2 -= 1);
                adjacent!("south", |pos: PosMut| pos.2 += 1);
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
                for details in &state.entities {
                    let id = details.id();
                    let entity_dir = reg.add_root_entry(
                        id.to_string(),
                        DirEntry::build()
                            .associated_data(EntryAssociatedData::EntityId(id))
                            .finish(),
                    );

                    mk_entity_dir(reg, entity_dir, EntityType::Other(details));
                }
            })
            .finish(),
    );
    dir
}

fn mk_entity_dir(reg: &mut DynamicDirRegistrationer, entity_dir: u64, ty: EntityType) {
    reg.add_entry(
        entity_dir,
        "health",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityHealth, Float))
            .filter(|state| state.is_in_game())
            .finish(),
    );
    reg.add_entry(
        entity_dir,
        "position",
        FileEntry::build()
            .behaviour(ReadWrite(CommandType::EntityPosition, Position))
            .filter(|state| state.is_in_game())
            .finish(),
    );

    reg.add_entry(
        entity_dir,
        "target",
        FileEntry::build()
            .behaviour(WriteOnly(CommandType::EntityTarget, Position))
            .filter(|state| state.is_in_game())
            .finish(),
    );

    if let EntityType::Other(details) = ty {
        reg.add_entry(
            entity_dir,
            "type",
            FileEntry::build()
                .behaviour(ReadOnly(CommandType::EntityType, String))
                .filter(|state| state.is_in_game())
                .finish(),
        );

        if details.living() {
            reg.add_entry(
                entity_dir,
                "living",
                FileEntry::build()
                    .behaviour(FileBehaviour::ForShow)
                    .filter(|state| state.is_in_game())
                    .finish(),
            );
        }
    }
}

// ------
fn parse_block_position(s: &str) -> Option<[i32; 3]> {
    let mut parts = s
        .splitn(3, &[',', ' '])
        .filter_map(|s| s.parse::<f32>().ok().map(|f| f as i32));

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
        assert!(matches!(
            parse_block_position("1.0 400.0 -64.0"),
            Some([1, 400, -64])
        ));
        assert!(matches!(
            parse_block_position("-4.011348385264139 105.89626455878891 56.17007141838458"),
            Some([-4, 105, 56])
        ));
        assert!(matches!(parse_block_position("0,0,0"), Some([0, 0, 0])));
        assert!(parse_block_position("oof").is_none());
        assert!(parse_block_position("1,2,3,4").is_none());
        assert!(parse_block_position("500,200").is_none());
        assert!(parse_block_position("123,nice,200").is_none());
    }
}
