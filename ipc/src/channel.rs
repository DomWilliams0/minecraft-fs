use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flatbuffers::{root, FlatBufferBuilder, InvalidFlatbuffer};
use log::warn;
use thiserror::Error;

use crate::command::{Body, BodyType, CommandState, TargetEntity};
use crate::generated::{
    BlockPos, CommandArgs, CommandType, Error, GameRequest, GameRequestArgs, GameRequestBody,
    GameResponse, GameResponseBody, StateRequest, StateRequestArgs, StateResponse, Vec3, WriteBody,
    WriteBodyArgs,
};

const RETRIES: u8 = 2;
const TIMEOUT: Duration = Duration::from_secs(5);

pub struct IpcChannel {
    sock_path: PathBuf,
    sock: UnixStream,
    retries: u8,
    recv_buffer: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Socket not found, game is probably not running")]
    NoGame,

    #[error("IO error connecting to socket, is the game running? ({0})")]
    Connecting(#[source] std::io::Error),

    #[cfg(feature = "client")]
    #[error("IO error binding to socket: {0}")]
    Binding(#[source] std::io::Error),

    #[error("IO error writing to socket: {0}")]
    Sending(#[source] std::io::Error),

    #[error("IO error reading from socket: {0}")]
    Receiving(#[source] std::io::Error),

    #[error("IO error setting socket timeout: {0}")]
    SettingTimeout(#[source] std::io::Error),

    #[error("Player is not currently in a game")]
    NoCurrentGame,

    #[error("Client error: {0}")]
    ClientError(&'static str),

    #[error("Got unexpected response type {0:?}")]
    UnexpectedGameResponse(GameResponseBody),

    #[error("Expected response type {0:?} but got something else")]
    UnexpectedResponse(Option<BodyType>),

    #[error("Deserialization failed: {0}")]
    Deserialization(#[from] InvalidFlatbuffer),

    #[error("Write data cannot be serialized into {0:?}")]
    BadData(BodyType),

    #[error("Invalid input")]
    BadInput,
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
        expected_response_type: Option<BodyType>,
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
                let mut block = None;
                match body {
                    Body::Integer(val) => int = Some(val),
                    Body::Float(val) => float = Some(val),
                    Body::String(val) => string = Some(buf.create_string(&val)),
                    Body::Vec { x, y, z } => pos = Some(Vec3::new(x, y, z)),
                    Body::Block { x, y, z } => block = Some(BlockPos::new(x, y, z)),
                }
                WriteBody::create(
                    &mut buf,
                    &WriteBodyArgs {
                        float,
                        int,
                        string,
                        vec: pos.as_ref(),
                        block: block.as_ref(),
                    },
                )
            });

            let (target_entity, target_player_entity) = match state.target_entity {
                Some(TargetEntity::Entity(id)) => (Some(id), false),
                Some(TargetEntity::Player) => (None, true),
                None => (None, false),
            };

            Command::create(
                &mut buf,
                &CommandArgs {
                    cmd,
                    target_entity,
                    target_player_entity,
                    target_world: state.target_world,
                    target_block: state.target_block.as_ref(),
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
                Error::BadInput => IpcError::BadInput,
                _ => IpcError::ClientError(err.variant_name().unwrap()),
            })
        } else {
            use BodyType::*;
            return Ok(Some(
                match (
                    expected_response_type,
                    response.float(),
                    response.int(),
                    response.string(),
                    response.vec(),
                ) {
                    (None, None, None, None, None) => return Ok(None),
                    (Some(Float), val, None, None, None) => Body::Float(val.unwrap_or(0.0)),
                    (Some(Integer), None, val, None, None) => Body::Integer(val.unwrap_or(0)),
                    (Some(String), None, None, Some(val), None) => Body::String(val.into()),
                    (Some(Position), None, None, None, Some(val)) => Body::Vec {
                        x: val.x(),
                        y: val.y(),
                        z: val.z(),
                    },
                    (expected, f, i, s, v) => {
                        warn!(
                            "expected {:?} but instead got this: float={:?},int={:?},str={:?},vec={:?}",
                            expected, f, i, s, v
                        );
                        return Err(IpcError::UnexpectedResponse(expected_response_type));
                    }
                },
            ));
        }
    }

    fn send_raw_request(&mut self, data: &[u8]) -> Result<(), IpcError> {
        let len = data.len() as u32;
        log::trace!("sending {} bytes on socket", len);
        #[cfg(feature = "log_socket")]
        log::trace!("data: {:02X?}", data);

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

        #[cfg(feature = "log_socket")]
        log::trace!("data: {:02X?}", self.recv_buffer);

        Ok(&self.recv_buffer)
    }

    fn open_socket(path: &Path) -> Result<UnixStream, IpcError> {
        match UnixStream::connect(path) {
            Ok(f) => {
                f.set_read_timeout(Some(TIMEOUT))
                    .map_err(IpcError::SettingTimeout)?;
                f.set_write_timeout(Some(TIMEOUT))
                    .map_err(IpcError::SettingTimeout)?;
                Ok(f)
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Err(IpcError::NoGame),
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

#[cfg(feature = "client")]
pub mod recv {
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};

    use flatbuffers::root;
    use log::*;

    use crate::generated::GameRequest;
    use crate::IpcError;

    /// Fake game client to the fuse server
    pub struct IpcClient {
        server: UnixListener,
    }

    pub struct ConnectedIpcClient {
        sock: UnixStream,
        recv_buffer: Vec<u8>,
    }

    impl IpcClient {
        pub fn bind() -> Result<Self, IpcError> {
            let path = {
                let user = std::env::var("USER").unwrap_or_else(|_| "user".to_owned());
                let mut path = std::env::temp_dir();
                path.push(format!("minecraft-fuse-{}", user));
                path
            };

            if path.exists() {
                if let Err(err) = std::fs::remove_file(&path) {
                    error!(
                        "failed to delete existing socket {}: {}",
                        path.display(),
                        err
                    )
                } else {
                    info!("deleted existing socket {}", path.display());
                }
            }

            info!("binding to socket {}", path.display());
            let server = UnixListener::bind(&path).map_err(IpcError::Binding)?;

            Ok(Self { server })
        }

        pub fn accept(&mut self) -> Result<ConnectedIpcClient, std::io::Error> {
            self.server.accept().map(|(s, _)| ConnectedIpcClient {
                sock: s,
                recv_buffer: vec![],
            })
        }
    }

    impl ConnectedIpcClient {
        pub fn recv(&mut self) -> Result<GameRequest, IpcError> {
            let mut len_bytes = [0u8; 4];
            self.sock
                .read_exact(&mut len_bytes)
                .map_err(IpcError::Receiving)?;
            let len = u32::from_le_bytes(len_bytes) as usize;
            debug!("recv'ing message of {} bytes", len);

            self.recv_buffer.truncate(0);
            self.recv_buffer.reserve_exact(len);

            {
                let dst_slice =
                    unsafe { std::slice::from_raw_parts_mut(self.recv_buffer.as_mut_ptr(), len) };

                self.sock
                    .read_exact(dst_slice)
                    .map_err(IpcError::Receiving)?;

                unsafe { self.recv_buffer.set_len(len) }
            }

            let req = root::<GameRequest>(&self.recv_buffer)?;
            Ok(req)
        }

        pub fn send_response(&mut self, response: &[u8]) -> Result<(), IpcError> {
            let len = response.len() as u32;
            log::trace!("sending {} bytes on socket", len);
            self.sock
                .write_all(&len.to_le_bytes())
                .map_err(IpcError::Sending)?;
            self.sock.write_all(response).map_err(IpcError::Sending)
        }
    }
}
