use ipc::generated::{Dimension, StateRequestArgs};
use ipc::{IpcChannel, IpcError};
use std::fmt::{Debug, Formatter};

use log::trace;
use std::time::{Duration, SystemTime};

const CACHE_TIME: Duration = Duration::from_millis(500);

#[derive(Default, Debug)]
pub struct GameState {
    pub player_entity_id: Option<i32>,
    pub player_world: Option<Dimension>,
    pub entity_ids: Vec<i32>,
}

pub struct CachedGameState {
    // TODO use Instant instead
    last_query: SystemTime,
    last_interest: GameStateInterest,
    state: GameState,
}

pub type GameStateInterest = StateRequestArgs;

impl Default for CachedGameState {
    fn default() -> Self {
        Self {
            last_query: SystemTime::now(),
            state: GameState::default(),
            last_interest: GameStateInterest::default(),
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
        let now = SystemTime::now();
        let stale = now
            .duration_since(self.last_query)
            .map(|d| d > CACHE_TIME)
            .unwrap_or(true);

        if stale || GameStateInterestWrapper(&self.last_interest).is_additive(&interest) {
            log::debug!(
                "sending state request with interest: {:?}",
                GameStateInterestWrapper(&interest)
            );
            let response = ipc.send_state_request(&interest)?;

            self.state = GameState {
                player_entity_id: response.player_entity_id(),
                player_world: response.player_world(),
                entity_ids: response
                    .entity_ids()
                    .map(|v| v.into_iter().collect())
                    .unwrap_or_default(),
            };
            trace!("new game state: {:?}", self.state);
            self.last_query = now;
            self.last_interest = interest;
        }

        Ok(&self.state)
    }
}

struct GameStateInterestWrapper<'a>(&'a GameStateInterest);

impl Debug for GameStateInterestWrapper<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameStateInterest")
            .field("entities_by_id", &self.0.entities_by_id)
            .field("target_world", &self.0.target_world)
            .finish()
    }
}

impl GameStateInterestWrapper<'_> {
    fn is_additive(&self, newer: &GameStateInterest) -> bool {
        if !self.0.entities_by_id && newer.entities_by_id {
            return true;
        }

        false
    }
}
