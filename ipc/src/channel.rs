use crate::command::{Command, Response, ResponseType};
use crate::CommandType;

use std::io::{ErrorKind, Read, Write};

use std::os::unix::net::UnixStream;
use thiserror::Error;

pub struct IpcChannel {
    sock: UnixStream,
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Socket not found, game is probably not running")]
    NotFound,

    #[error("IO error connecting to socket: {0}")]
    Connecting(#[source] std::io::Error),

    #[error("IO error sending command: {0}")]
    SendingCommand(#[source] std::io::Error),

    #[error("IO error reading response: {0}")]
    ReadingResponse(#[source] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("Deserialization error: {0}")]
    Deserialize(#[source] serde_json::Error),

    #[error("Unexpected response type, expected {0:?} but got {1:?}")]
    BadResponse(ResponseType, serde_json::Value),
}

impl IpcChannel {
    pub fn open_existing() -> Result<Self, IpcError> {
        let path = {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_owned());
            let mut path = std::env::temp_dir();
            path.push(format!("minecraft-fuse-{}", user));
            path
        };

        log::debug!("opening domain socket {}", path.display());
        let sock = match UnixStream::connect(path) {
            Ok(f) => f,
            Err(err) if err.kind() == ErrorKind::NotFound => return Err(IpcError::NotFound),
            Err(err) => return Err(IpcError::Connecting(err)),
        };

        Ok(Self { sock })
    }

    pub fn send_command(&mut self, command_type: CommandType) -> Result<Response, IpcError> {
        // TODO reuse buffer allocation
        let mut buf = Vec::new();
        let command = Command { ty: command_type };
        serde_json::to_writer(&mut buf, &command).map_err(IpcError::Serialize)?;

        {
            let len = buf.len() as u32;
            log::trace!("sending {} bytes for command {:?}", len, command);
            self.sock
                .write_all(&len.to_be_bytes())
                .map_err(IpcError::SendingCommand)?;

            self.sock
                .write_all(&buf)
                .map_err(IpcError::SendingCommand)?;
        }

        let resp_type = match command_type.response_type() {
            Some(ty) => ty,
            None => return Ok(Response::None),
        };

        buf.clear();
        {
            let mut len_bytes = [0u8; 4];
            self.sock
                .read_exact(&mut len_bytes)
                .map_err(IpcError::ReadingResponse)?;

            let len = u32::from_be_bytes(len_bytes);
            log::trace!("reading {} bytes from socket", len);

            buf.resize(len as usize, 0);
            self.sock
                .read_exact(&mut buf)
                .map_err(IpcError::ReadingResponse)?;
        }

        let json: serde_json::Value =
            serde_json::from_slice(&buf).map_err(IpcError::Deserialize)?;

        // TODO specific errors like no such player
        log::trace!("response is {:?}", json);

        match (resp_type, json) {
            (ResponseType::Integer, serde_json::Value::Number(n)) if n.is_i64() => {
                Ok(Response::Integer(n.as_i64().unwrap()))
            }
            (ResponseType::Float, serde_json::Value::Number(n)) if n.is_f64() => {
                Ok(Response::Float(n.as_f64().unwrap()))
            }
            (ResponseType::String, serde_json::Value::String(s)) => Ok(Response::String(s)),
            (expected, got) => Err(IpcError::BadResponse(expected, got)),
        }
    }
}
