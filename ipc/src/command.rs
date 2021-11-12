use crate::generated::CommandType;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum ResponseType {
    Integer,
    String,
    Float,
    Position,
}

pub enum ReadCommand {
    WithResponse(CommandType, ResponseType),
    // TODO WithoutResponse?
}

pub enum ResponseBody {
    None,
    Integer(i32),
    Float(f32),
    String(String),
    Position { x: f64, y: f64, z: f64 },
}

impl Display for ResponseBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseBody::None => Ok(()),
            ResponseBody::Float(val) => Debug::fmt(val, f),
            ResponseBody::Integer(val) => Display::fmt(val, f),
            ResponseBody::String(val) => Display::fmt(val, f),
            ResponseBody::Position { x, y, z } => write!(f, "{:?}\n{:?}\n{:?}", x, y, z),
        }
    }
}
