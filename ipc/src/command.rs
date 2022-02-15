use crate::generated::{BlockPos, CommandType, Dimension};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum BodyType {
    Integer,
    String,
    Float,
    Position,
}

pub enum Body<'a> {
    Integer(i32),
    Float(f32),
    String(Cow<'a, str>),
    Vec { x: f64, y: f64, z: f64 },
    Block { x: i32, y: i32, z: i32 },
}

pub enum TargetEntity {
    Player,
    Entity(i32),
}

#[derive(Default)]
pub struct CommandState {
    pub target_entity: Option<TargetEntity>,
    pub target_world: Option<Dimension>,
    pub target_block: Option<BlockPos>,
}

pub struct Command {
    pub ty: CommandType,
    pub state: CommandState,
    /// Expected response type for a read, or request body type for a write
    pub body: BodyType,
}

impl Display for Body<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::Float(val) => Debug::fmt(val, f),
            Body::Integer(val) => Display::fmt(val, f),
            Body::String(val) => Display::fmt(val, f),
            Body::Vec { x, y, z } => write!(f, "{:?} {:?} {:?}", x, y, z),
            Body::Block { x, y, z } => write!(f, "{:?} {:?} {:?}", x, y, z),
        }
    }
}

impl Command {
    pub fn stateful(cmd: CommandType, resp: BodyType, state: CommandState) -> Self {
        Self {
            ty: cmd,
            state,
            body: resp,
        }
    }

    pub fn stateless(cmd: CommandType, resp: BodyType) -> Self {
        Self::stateful(cmd, resp, CommandState::default())
    }
}

impl BodyType {
    pub fn create_from_data<'a>(&self, data: &'a [u8]) -> Option<Body<'a>> {
        let data = std::str::from_utf8(data).ok()?.trim_end();
        match self {
            BodyType::Float => data.parse().ok().map(Body::Float),
            BodyType::Integer => data.parse().ok().map(Body::Integer),
            BodyType::String => Some(Body::String(data.into())),
            BodyType::Position => {
                let xyz = data.split_whitespace();
                let mut iter = xyz.into_iter().take(3).map(|s| s.parse());

                if let (Some(Ok(x)), Some(Ok(y)), Some(Ok(z))) =
                    (iter.next(), iter.next(), iter.next())
                {
                    Some(Body::Vec { x, y, z })
                } else {
                    None
                }
            }
        }
    }
}
