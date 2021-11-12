use ipc::generated::StateRequestArgs;
use ipc::{IpcChannel, IpcError};

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

const CACHE_TIME: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct GameState {
    pub is_in_game: bool,
}

/// FS registers its interest in its reference of this
#[derive(Default)]
pub struct GameStateInterest {
    pub entity_list: bool,
}

pub struct CachedGameState {
    last_query: SystemTime,
    interest: Arc<Mutex<GameStateInterest>>,
    state: GameState,
}

impl CachedGameState {
    pub fn new(interest: Arc<Mutex<GameStateInterest>>) -> Self {
        Self {
            last_query: SystemTime::now(),
            state: GameState::default(),
            interest,
        }
    }

    pub fn get(&mut self, ipc: &mut IpcChannel) -> Result<&GameState, IpcError> {
        let now = SystemTime::now();
        let stale = now
            .duration_since(self.last_query)
            .map(|d| d > CACHE_TIME)
            .unwrap_or(true);

        if stale {
            let interest = self.interest.lock();
            let req = StateRequestArgs {
                entity_list: interest.entity_list,
            };

            let response = ipc.send_state_request(&req)?;

            self.state = GameState {
                is_in_game: response.is_in_game(),
            }
        }

        Ok(&self.state)
    }
}
