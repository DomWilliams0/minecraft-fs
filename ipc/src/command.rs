use serde::Serialize;
use std::fmt::{Display, Formatter};

// TODO define as a protobuf for use in plugin/mod

#[derive(Serialize, Copy, Clone, Debug)]
#[repr(u8)]
pub enum CommandType {
    PlayerHealth,
}

#[derive(Serialize, Debug)]
pub struct Command {
    pub ty: CommandType,
}

#[derive(Debug)]
pub enum ResponseType {
    Integer,
    String,
    Float,
}

pub enum Response {
    None,
    Integer(i64),
    Float(f64),
    String(String),
    Json(serde_json::Value),
}

impl CommandType {
    pub fn response_type(&self) -> Option<ResponseType> {
        use CommandType::*;
        use ResponseType::*;
        Some(match self {
            PlayerHealth => Float,
        })
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Response::None => return Ok(()),
            Response::Integer(val) => val as &dyn Display,
            Response::Float(val) => val as &dyn Display,
            Response::String(val) => val as &dyn Display,
            Response::Json(val) => val as &dyn Display,
        };

        Display::fmt(display, f)
    }
}
