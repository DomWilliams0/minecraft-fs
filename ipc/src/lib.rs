mod channel;
mod command;
pub mod generated;

pub use channel::{IpcChannel, IpcError};
pub use command::{Body, BodyType, Command, CommandState, TargetEntity};
