use arena_core::{Action, Config, Outcome, Turn, View};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRoom {
    pub nickname: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JoinRoom {
    pub nickname: String,
}

/// A stable room identity, independent of seating. Keep its bearer token private.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Membership {
    pub code: String,
    pub member: usize,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemberView {
    pub id: usize,
    pub nickname: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameSnapshot {
    pub id: String,
    pub config: Config,
    /// Members in immutable game-player order.
    pub players: Vec<MemberView>,
    /// The requesting member's game-player index; null on the bench.
    pub you: Option<usize>,
    pub can_finish: bool,
    pub turn: Turn,
    pub outcome: Option<Outcome>,
    /// Benched members receive no game observation.
    pub view: Option<View>,
    pub legal_actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReplaySummary {
    pub id: String,
    pub config: Config,
    pub players: Vec<MemberView>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Snapshot {
    pub code: String,
    pub members: Vec<MemberView>,
    pub host: usize,
    pub you: usize,
    pub revision: u64,
    /// Selected setup options; null until the host chooses a game.
    pub config: Option<Config>,
    /// Members selected for the next game, in player order. Others are benched.
    pub seating: Vec<usize>,
    pub game: Option<GameSnapshot>,
    pub replays: Vec<ReplaySummary>,
}

/// Messages routed to the identified game. Host controls are not player moves.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameMessage {
    Move {
        action: Action,
    },
    /// Records The Crew's shared result; only the room host may submit it.
    Finish {
        won: bool,
    },
}

/// Authenticate in the first frame. Setup commands require the exact revision;
/// game messages allow older same-game revisions and are revalidated FIFO.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    Authenticate {
        token: String,
    },
    Configure {
        revision: u64,
        config: Option<Config>,
    },
    Seat {
        revision: u64,
        members: Vec<usize>,
    },
    Start {
        revision: u64,
    },
    Abort {
        revision: u64,
        game_id: String,
    },
    Game {
        revision: u64,
        game_id: String,
        message: GameMessage,
    },
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Snapshot {
        room: Box<Snapshot>,
        /// True only for the submitting socket's successful command response.
        acknowledged: bool,
    },
    Error {
        error: String,
    },
}
