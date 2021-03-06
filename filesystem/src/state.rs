use std::fmt::Debug;
use std::time::{Duration, Instant};

use log::{debug, trace};

use ipc::generated::{BlockPos, Dimension, EntityDetails, StateRequestArgs};
use ipc::{IpcChannel, IpcError};

const CACHE_TIME: Duration = Duration::from_millis(500);

#[derive(Default, Debug)]
pub struct GameState {
    pub player_entity_id: Option<i32>,
    pub player_world: Option<Dimension>,
    pub entities: Vec<EntityDetails>,
    pub block: Option<BlockDetails>,
}

#[derive(Debug)]
pub struct BlockDetails {
    pub pos: BlockPos,
}

pub struct CachedGameState {
    last_query: Instant,
    last_interest: GameStateInterest,
    state: GameState,
}

/// Maps to generated `StateRequestArgs`
#[derive(Default, Debug)]
pub struct GameStateInterest {
    pub entities_by_id: bool,
    pub target_world: Option<Dimension>,
    pub target_block: Option<BlockPos>,
}

impl Default for CachedGameState {
    fn default() -> Self {
        Self {
            last_query: Instant::now(),
            state: GameState::default(),
            last_interest: GameStateInterest::default(),
        }
    }
}

impl GameStateInterest {
    pub fn as_state_request_args(&self) -> StateRequestArgs {
        StateRequestArgs {
            entities_by_id: self.entities_by_id,
            target_world: self.target_world,
            target_block: self.target_block.as_ref(),
        }
    }
}

impl GameState {
    pub fn is_in_game(&self) -> bool {
        self.player_entity_id.is_some()
    }
}

impl CachedGameState {
    pub fn get(
        &mut self,
        ipc: &mut IpcChannel,
        interest: GameStateInterest,
    ) -> Result<&GameState, IpcError> {
        let now = Instant::now();
        let stale = now.duration_since(self.last_query) > CACHE_TIME;

        log::debug!("getting state for interest: {:?}", interest);
        let additive = self.last_interest.is_additive(&interest);
        if stale || additive {
            if stale {
                trace!("old state is stale");
            }
            if additive {
                trace!(
                    "new interest is additive to old interest: {:?}",
                    self.last_interest
                )
            }
            debug!("sending state request");

            let response = ipc.send_state_request(&interest.as_state_request_args())?;

            self.state = GameState {
                player_entity_id: response.player_entity_id(),
                player_world: response.player_world(),
                entities: response
                    .entities()
                    .map(|v| v.iter().copied().collect())
                    .unwrap_or_default(),
                block: response.block().map(|b| BlockDetails { pos: *b.pos() }),
            };
            trace!("new game state: {:?}", self.state);
            self.last_query = now;
            self.last_interest = interest;
        } else {
            debug!("using cached state for interest");
            trace!("previous interest: {:?}", self.last_interest);
        }

        Ok(&self.state)
    }
}

impl GameStateInterest {
    fn is_additive(&self, newer: &GameStateInterest) -> bool {
        if !self.entities_by_id && newer.entities_by_id {
            return true;
        }

        if newer.target_block.is_some() {
            // only bother checking if we now care about target block
            if self.target_block != newer.target_block {
                return true;
            }
        }

        false
    }
}
