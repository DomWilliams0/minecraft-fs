use std::io::{ErrorKind, Read, Write};

use crate::command::{Body, BodyType, CommandState};
use crate::generated::{
    CommandArgs, CommandType, Error, GameRequest, GameRequestArgs, GameRequestBody, GameResponse,
    GameResponseBody, StateRequest, StateRequestArgs, StateResponse, Vec3, WriteBody,
    WriteBodyArgs,
};

use flatbuffers::{root, FlatBufferBuilder, InvalidFlatbuffer};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use thiserror::Error;

const RETRIES: u8 = 2;

pub struct IpcChannel {
    sock_path: PathBuf,
    sock: UnixStream,
    retries: u8,
    recv_buffer: Vec<u8>,
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

    #[error("Got unexpected response type {0:?}")]
    UnexpectedGameResponse(GameResponseBody),

    #[error("Expected response type {0:?} but got something else")]
    UnexpectedResponse(BodyType),

    #[error("Deserialization failed: {0}")]
    Deserialization(#[from] InvalidFlatbuffer),

    #[error("Write data cannot be serialized into {0:?}")]
    BadData(BodyType),
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
            recv_buffer: Vec::with_capacity(8192),
        })
    }

    pub fn send_read_command(
        &mut self,
        cmd: CommandType,
        resp: BodyType,
        state: CommandState,
    ) -> Result<Body, IpcError> {
        self.send_raw_command(cmd, Some(resp), None, state)
            .map(|opt| opt.expect("response expected"))
    }

    pub fn send_write_command(
        &mut self,
        cmd: CommandType,
        body_type: BodyType,
        data: &[u8],
        state: CommandState,
    ) -> Result<usize, IpcError> {
        log::trace!("write data {:?}", data);
        let write = body_type
            .create_from_data(data)
            .ok_or(IpcError::BadData(body_type))?;

        self.send_raw_command(cmd, None, Some(write), state)?;
        Ok(data.len())
    }

    pub fn send_state_request(
        &mut self,
        req: &StateRequestArgs,
    ) -> Result<StateResponse, IpcError> {
        // TODO reuse buffer allocation
        let mut buf = FlatBufferBuilder::with_capacity(1024);

        let req = StateRequest::create(&mut buf, req);
        let req = GameRequest::create(
            &mut buf,
            &GameRequestArgs {
                body_type: GameRequestBody::StateRequest,
                body: Some(req.as_union_value()),
            },
        );
        buf.finish(req, None);

        self.send_raw_request(buf.finished_data())?;

        let response = self
            .recv_raw_response()
            .and_then(|resp| root::<GameResponse>(resp).map_err(IpcError::Deserialization))?;

        response
            .body_as_state_response()
            .ok_or_else(|| IpcError::UnexpectedGameResponse(response.body_type()))
    }

    fn send_raw_command(
        &mut self,
        cmd: CommandType,
        response_type: Option<BodyType>,
        write: Option<Body>,
        state: CommandState,
    ) -> Result<Option<Body>, IpcError> {
        use crate::generated::Command;

        // TODO reuse buffer allocation
        let mut buf = FlatBufferBuilder::with_capacity(1024);
        let cmd = {
            let write_body = write.map(|body| {
                let mut float = None;
                let mut int = None;
                let mut string = None;
                let mut pos = None;
                match body {
                    Body::Integer(val) => int = Some(val),
                    Body::Float(val) => float = Some(val),
                    Body::String(val) => string = Some(buf.create_string(&val)),
                    Body::Position { x, y, z } => pos = Some(Vec3::new(x, y, z)),
                }
                WriteBody::create(
                    &mut buf,
                    &WriteBodyArgs {
                        float,
                        int,
                        string,
                        pos: pos.as_ref(),
                    },
                )
            });

            Command::create(
                &mut buf,
                &CommandArgs {
                    cmd,
                    target_entity: state.target_entity,
                    target_world: state.target_world,
                    write: write_body,
                },
            )
        };

        let req = GameRequest::create(
            &mut buf,
            &GameRequestArgs {
                body_type: GameRequestBody::Command,
                body: Some(cmd.as_union_value()),
            },
        );
        buf.finish(req, None);

        self.send_raw_request(buf.finished_data())?;

        let resp_type = match response_type {
            Some(ty) => ty,
            None => return Ok(None),
        };

        let response = self
            .recv_raw_response()
            .and_then(|resp| root::<GameResponse>(resp).map_err(IpcError::Deserialization))?;

        let response = match response.body_as_response() {
            Some(resp) => resp,
            None => return Err(IpcError::UnexpectedGameResponse(response.body_type())),
        };

        if let Some(err) = response.error() {
            Err(match err {
                Error::NoGame => IpcError::NoCurrentGame,
                _ => IpcError::ClientError(err.variant_name().unwrap()),
            })
        } else {
            use BodyType::*;
            return Ok(Some(
                match (
                    resp_type,
                    response.float(),
                    response.int(),
                    response.string(),
                    response.pos(),
                ) {
                    (Float, Some(val), None, None, None) => Body::Float(val),
                    (Integer, None, Some(val), None, None) => Body::Integer(val),
                    (String, None, None, Some(val), None) => Body::String(val.into()),
                    (Position, None, None, None, Some(val)) => Body::Position {
                        x: val.x(),
                        y: val.y(),
                        z: val.z(),
                    },
                    _ => return Err(IpcError::UnexpectedResponse(resp_type)),
                },
            ));
        }
    }

    fn send_raw_request(&mut self, data: &[u8]) -> Result<(), IpcError> {
        let len = data.len() as u32;
        log::trace!("sending {} bytes on socket", len);
        self.attempt_write(&len.to_le_bytes())?;
        self.attempt_write(data)
    }

    fn recv_raw_response(&mut self) -> Result<&[u8], IpcError> {
        let mut len_bytes = [0u8; 4];
        self.sock
            .read_exact(&mut len_bytes)
            .map_err(IpcError::Receiving)?;

        let len = u32::from_le_bytes(len_bytes);
        log::trace!("reading {} bytes from socket", len);

        self.recv_buffer.resize(len as usize, 0);
        self.sock
            .read_exact(&mut self.recv_buffer)
            .map_err(IpcError::Receiving)?;

        Ok(&self.recv_buffer)
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
                Ok(_) => {
                    self.retries = RETRIES;
                    return Ok(());
                }
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
