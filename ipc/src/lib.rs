mod channel;
mod command;
pub mod generated;

#[cfg(feature = "client")]
pub use channel::recv::{ConnectedIpcClient, IpcClient};
pub use channel::{IpcChannel, IpcError};
pub use command::{Body, BodyType, Command, CommandState, TargetEntity};
