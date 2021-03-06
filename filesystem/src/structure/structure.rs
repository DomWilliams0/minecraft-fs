use std::borrow::Cow;

use crate::state::GameState;
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

fn filter_in_game(state: &GameState) -> bool {
    state.is_in_game()
}

#[allow(unused_variables)]
pub fn create_structure() -> FilesystemStructure {
    let mut builder = FilesystemStructure::builder();

    player_dir(&mut builder);
    worlds_dir(&mut builder);

    builder.add_entry(
        builder.root(),
        "version",
        FileEntry::build(FileBehaviour::Static(Cow::Borrowed(env!(
            "CARGO_PKG_VERSION"
        ))))
        .finish(),
    );

    builder.add_entry(
        builder.root(),
        "command",
        FileEntry::build(FileBehaviour::WriteOnly(CommandType::ServerCommand, String))
            .filter(filter_in_game)
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
            .dynamic(DynamicStateType::PlayerId, |_, reg| {
                mk_entity_dir(reg, reg.parent(), EntityType::SpecificallyPlayer);
            })
            .finish(),
    );
    builder.add_entry(
        dir,
        "name",
        FileEntry::build(FileBehaviour::ReadOnly(CommandType::PlayerName, String)).finish(),
    );

    builder.add_entry(
        dir,
        "gamemode",
        FileEntry::build(FileBehaviour::ReadWrite(
            CommandType::PlayerGamemode,
            String,
        ))
        .filter(filter_in_game)
        .finish(),
    );

    builder.add_entry(
        dir,
        "hunger",
        FileEntry::build(FileBehaviour::ReadWrite(CommandType::PlayerHunger, Integer))
            .filter(filter_in_game)
            .finish(),
    );

    builder.add_entry(
        dir,
        "saturation",
        FileEntry::build(FileBehaviour::ReadWrite(
            CommandType::PlayerSaturation,
            Float,
        ))
        .filter(filter_in_game)
        .finish(),
    );
    builder.add_entry(
        dir,
        "exhaustion",
        FileEntry::build(FileBehaviour::ReadWrite(
            CommandType::PlayerExhaustion,
            Float,
        ))
        .filter(filter_in_game)
        .finish(),
    );

    builder.add_entry(
        dir,
        "entity",
        LinkEntry::build(|state| {
            Some(format!("world/entities/by-id/{}", state.player_entity_id?).into())
        })
        .filter(filter_in_game)
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
        .filter(filter_in_game)
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
        FileEntry::build(FileBehaviour::WriteOnly(
            CommandType::ControlSay,
            BodyType::String,
        ))
        .finish(),
    );

    builder.add_entry(
        control,
        "jump",
        FileEntry::build(FileBehaviour::WriteOnly(
            CommandType::ControlJump,
            BodyType::String,
        ))
        .finish(),
    );

    builder.add_entry(
        control,
        "move",
        FileEntry::build(FileBehaviour::WriteOnly(
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
            FileEntry::build(FileBehaviour::ReadWrite(CommandType::WorldTime, Integer)).finish(),
        );

        let blocks_dir = builder.add_entry(world, "blocks", DirEntry::default());
        builder.add_entry(
            blocks_dir,
            "README",
            FileEntry::build(FileBehaviour::Static(
                "Path format is ./x,y,z or ./x\\ y\\ z\ne.g. 0,64,100 or \"0.5 22.3 41.5555\"\n"
                    .into(),
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
                    FileEntry::build(FileBehaviour::ReadWrite(
                        CommandType::BlockType,
                        BodyType::String,
                    ))
                    .finish(),
                );

                reg.add_root_entry(
                    "pos",
                    FileEntry::build(FileBehaviour::Static(
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

    builder.add_entry(
        dir,
        "spawn",
        FileEntry::build(FileBehaviour::CommandProxy {
            readme: r#"Spawn an entity at a position with optional NBT tags.
Accepts same position as /summon command (e.g. ~20 100 ~5).
Format: "[entity type]\n[position]\n<nbt>"
Examples:
   pig\n100 20 50\n
   chicken\n20.0 64.5 100.0\n
   creeper\n0,64,0\n{powered:1b,CustomName:'{"text":"Powered Creeper"}'}"#
                .into(),
            produce_cmd_fn: (|input| {
                let (entity_ty, pos, nbt) = {
                    let mut lines = input.lines();
                    let entity_ty = lines.next()?; // required
                    let pos = lines.next()?; // required, let the server parse it
                    let nbt = lines.next().unwrap_or_default(); // optional
                    (entity_ty, pos, nbt)
                };

                Some(format!("summon {entity_ty} {pos} {nbt}"))
            }),
        })
        .finish(),
    );

    dir
}

fn mk_entity_dir(reg: &mut DynamicDirRegistrationer, entity_dir: u64, ty: EntityType) {

    reg.add_entry(
        entity_dir,
        "position",
        FileEntry::build(ReadWrite(CommandType::EntityPosition, Position))
            .filter(filter_in_game)
            .finish(),
    );

    reg.add_entry(
        entity_dir,
        "target",
        FileEntry::build(WriteOnly(CommandType::EntityTarget, Position))
            .filter(filter_in_game)
            .finish(),
    );

    let add_health = match ty {
        EntityType::SpecificallyPlayer => true,
        EntityType::Other(details) => details.living()
    };

    if add_health {
        reg.add_entry(
            entity_dir,
            "health",
            FileEntry::build(ReadWrite(CommandType::EntityHealth, Float))
                .filter(filter_in_game)
                .finish(),
        );
    }

    if let EntityType::Other(details) = ty {
        reg.add_entry(
            entity_dir,
            "type",
            FileEntry::build(ReadOnly(CommandType::EntityType, String))
                .filter(filter_in_game)
                .finish(),
        );

        if details.living() {
            reg.add_entry(
                entity_dir,
                "living",
                FileEntry::build(FileBehaviour::ForShow)
                    .filter(filter_in_game)
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
