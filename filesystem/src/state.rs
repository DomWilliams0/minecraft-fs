use ipc::generated::StateRequestArgs;
use ipc::{IpcChannel, IpcError};

use std::time::{Duration, SystemTime};

const CACHE_TIME: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct GameState {
    pub is_in_game: bool,
}

pub struct CachedGameState {
    last_query: SystemTime,
    state: GameState,
}

pub type GameStateInterest = StateRequestArgs;

impl Default for CachedGameState {
    fn default() -> Self {
        Self {
            last_query: SystemTime::now(),
            state: GameState::default(),
        }
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

        if stale {
            log::debug!("sending state request");
            let response = ipc.send_state_request(&interest)?;

            self.state = GameState {
                is_in_game: response.is_in_game(),
            };
            self.last_query = now;
        }

        Ok(&self.state)
    }
}
