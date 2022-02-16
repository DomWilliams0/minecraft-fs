use std::borrow::Cow;
use std::error::Error as StdError;
use std::iter::once;

use flatbuffers::FlatBufferBuilder;
use log::*;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};

use ipc::generated::{
    BlockDetails, BlockDetailsArgs, Command, CommandType, Dimension, EntityDetails, Error,
    GameResponse, GameResponseArgs, GameResponseBody, Response, ResponseArgs, StateResponse,
    StateResponseArgs, Vec3,
};
use ipc::{ConnectedIpcClient, IpcClient, IpcError};

/// Client to fuse server
struct Client {
    channel: IpcClient,
}

impl Client {
    fn new() -> Result<Self, IpcError> {
        IpcClient::bind().map(|channel| Self { channel })
    }
}

enum ClientCommandResponse {
    Error(Error),
    Float(f32),
    Int(i32),
    String(Cow<'static, str>),
    Vec(Vec3),
}

enum ClientResponse {
    Command(ClientCommandResponse),
    State { requested_block: bool },
}

fn target_entity(cmd: &Command) -> Result<i32, Error> {
    cmd.target_entity()
        .or_else(|| {
            if cmd.target_player_entity() {
                Some(0)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            warn!("missing target entity");
            Error::MalformedRequest
        })
}

fn handle_client(mut client: ConnectedIpcClient) -> Result<(), Box<dyn StdError>> {
    let mut buf = FlatBufferBuilder::with_capacity(1024);
    loop {
        let msg = client.recv()?;
        debug!("handling msg '{:?}'", msg);

        let resp_body_type;
        let resp = if let Some(cmd) = msg.body_as_command() {
            if cmd.write().is_some() {
                // no response for write commands
                continue;
            }

            resp_body_type = GameResponseBody::Response;
            ClientResponse::Command(match cmd.cmd() {
                CommandType::PlayerName => ClientCommandResponse::String("TestPlayer".into()),
                CommandType::EntityType => ClientCommandResponse::String("Cow".into()),
                CommandType::EntityPosition => {
                    ClientCommandResponse::Vec(Vec3::new(100.0, 64.0, 205.2))
                }
                CommandType::EntityHealth => match target_entity(&cmd) {
                    Ok(_) => ClientCommandResponse::Float(10.0),
                    Err(err) => ClientCommandResponse::Error(err),
                },
                CommandType::WorldTime => ClientCommandResponse::Int(500),
                CommandType::ControlSay | CommandType::ControlJump | CommandType::ControlMove => {
                    continue
                }
                _ => ClientCommandResponse::Error(Error::UnknownCommand),
            })
        } else if let Some(req) = msg.body_as_state_request() {
            resp_body_type = GameResponseBody::StateResponse;
            ClientResponse::State {
                requested_block: req.target_world().is_some() && req.target_block().is_some(),
            }
        } else {
            unreachable!("bad msg type") // TODO send error?
        };

        let resp_body = match resp {
            ClientResponse::Command(resp) => {
                let mut body = ResponseArgs::default();
                match &resp {
                    ClientCommandResponse::Error(val) => body.error = Some(*val),
                    ClientCommandResponse::Float(val) => body.float = Some(*val),
                    ClientCommandResponse::Int(val) => body.int = Some(*val),
                    ClientCommandResponse::String(val) => {
                        body.string = Some(buf.create_string(val));
                    }
                    ClientCommandResponse::Vec(val) => body.vec = Some(val),
                }
                Response::create(&mut buf, &body).as_union_value()
            }
            ClientResponse::State { requested_block } => {
                let block = if requested_block {
                    Some(BlockDetails::create(
                        &mut buf,
                        &BlockDetailsArgs { has_color: true },
                    ))
                } else {
                    None
                };

                let mut rand = thread_rng();
                let n = rand.gen_range(3..10);
                let entities = once(EntityDetails::new(0, true, true))
                    .chain((1usize..n).map(|_| {
                        EntityDetails::new(
                            rand.gen_range(1..100),
                            rand.gen_bool(0.5),
                            rand.gen_bool(0.9),
                        )
                    }))
                    .collect::<Vec<_>>();

                let state = StateResponseArgs {
                    player_entity_id: Some(0),
                    player_world: Some(Dimension::Overworld),
                    entities: Some(buf.create_vector_direct(&entities)),
                    block,
                };
                StateResponse::create(&mut buf, &state).as_union_value()
            }
        };

        let resp_root = GameResponse::create(
            &mut buf,
            &GameResponseArgs {
                body_type: resp_body_type,
                body: Some(resp_body),
            },
        );

        buf.finish(resp_root, None);
        client.send_response(buf.finished_data())?;

        buf.reset();
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::init();

    let mut client = Client::new()?;

    loop {
        let connected = match client.channel.accept() {
            Ok(c) => {
                debug!("client connected");
                c
            }
            Err(err) => {
                error!("connection failure: {}", err);
                continue;
            }
        };

        match handle_client(connected) {
            Ok(_) => {}
            Err(err) => {
                error!("error handling client: {}", err);
                continue;
            }
        }
    }
}
