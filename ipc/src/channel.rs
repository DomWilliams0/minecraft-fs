use std::io::{ErrorKind, Read, Write};

use crate::command::{ResponseBody, ResponseType};
use crate::generated::{root_as_response, Command, CommandArgs, CommandType, Error};
use crate::ReadCommand;
use flatbuffers::FlatBufferBuilder;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use thiserror::Error;

const RETRIES: u8 = 2;

pub struct IpcChannel {
    sock_path: PathBuf,
    sock: UnixStream,
    retries: u8,
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Socket not found, game is probably not running")]
    NotFound,

    #[error("IO error connecting to socket: {0}")]
    Connecting(#[source] std::io::Error),

    #[error("IO error writing to socket: {0}")]
    Sending(#[source] std::io::Error),

    #[error("IO error reading from socket: {0}")]
    Receiving(#[source] std::io::Error),

    #[error("Player is not currently in a game")]
    NoCurrentGame,

    #[error("Client error: {0}")]
    ClientError(&'static str),

    #[error("Expected response type {0:?} but got something else")]
    UnexpectedResponse(ResponseType),
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
        let sock = Self::open_socket(&path)?;

        Ok(Self {
            sock_path: path,
            sock,
            retries: RETRIES,
        })
    }

    pub fn send_read(&mut self, cmd: ReadCommand) -> Result<ResponseBody, IpcError> {
        let (cmd, resp) = match cmd {
            ReadCommand::WithResponse(cmd, resp) => (cmd, Some(resp)),
        };

        self.send_command(cmd, resp)
    }

    fn send_command(
        &mut self,
        command: CommandType,
        response_type: Option<ResponseType>,
    ) -> Result<ResponseBody, IpcError> {
        // TODO reuse buffer allocation
        let mut recv_buffer = Vec::with_capacity(8192);
        let mut buf = FlatBufferBuilder::with_capacity(1024);
        {
            let offset = Command::create(&mut buf, &CommandArgs { cmd: command });
            buf.finish(offset, None);
        }

        {
            let data = buf.finished_data();
            let len = data.len() as u32;
            log::trace!("sending {} bytes for command {:?}", len, command);
            self.attempt_write(&len.to_le_bytes())?;
            self.attempt_write(data)?;
        }

        let resp_type = match response_type {
            Some(ty) => ty,
            None => return Ok(ResponseBody::None),
        };

        {
            let mut len_bytes = [0u8; 4];
            self.sock
                .read_exact(&mut len_bytes)
                .map_err(IpcError::Receiving)?;

            let len = u32::from_le_bytes(len_bytes);
            log::trace!("reading {} bytes from socket", len);

            recv_buffer.resize(len as usize, 0);
            self.sock
                .read_exact(&mut recv_buffer)
                .map_err(IpcError::Receiving)?;
        }

        let response = root_as_response(&recv_buffer).expect("bad");

        if let Some(err) = response.error() {
            Err(match err {
                Error::NoGame => IpcError::NoCurrentGame,
                _ => IpcError::ClientError(err.variant_name().unwrap()),
            })
        } else {
            match (
                resp_type,
                response.float(),
                response.int(),
                response.string(),
                response.pos(),
            ) {
                (ResponseType::Float, Some(val), None, None, None) => Ok(ResponseBody::Float(val)),
                (ResponseType::Integer, None, Some(val), None, None) => {
                    Ok(ResponseBody::Integer(val))
                }
                // TODO dont clone string
                (ResponseType::String, None, None, Some(val), None) => {
                    Ok(ResponseBody::String(val.to_owned()))
                }
                (ResponseType::Position, None, None, None, Some(val)) => {
                    Ok(ResponseBody::Position {
                        x: val.x(),
                        y: val.y(),
                        z: val.z(),
                    })
                }
                _ => Err(IpcError::UnexpectedResponse(resp_type)),
            }
        }
    }

    fn open_socket(path: &Path) -> Result<UnixStream, IpcError> {
        match UnixStream::connect(path) {
            Ok(f) => Ok(f),
            Err(err) if err.kind() == ErrorKind::NotFound => Err(IpcError::NotFound),
            Err(err) => Err(IpcError::Connecting(err)),
        }
    }

    fn attempt_write(&mut self, data: &[u8]) -> Result<(), IpcError> {
        fn rebootable(kind: ErrorKind) -> bool {
            matches!(kind, ErrorKind::BrokenPipe | ErrorKind::ConnectionRefused)
        }

        loop {
            match self.sock.write_all(data) {
                Ok(_) => return Ok(()),
                Err(err) if !rebootable(err.kind()) || self.retries == 0 => {
                    return Err(IpcError::Sending(err))
                }
                Err(_) => {
                    self.retries -= 1;

                    // reboot and try again
                    log::debug!("reopening socket, {} retries remaining", self.retries);
                    self.sock = Self::open_socket(&self.sock_path)?;
                }
            }
        }
    }
}
