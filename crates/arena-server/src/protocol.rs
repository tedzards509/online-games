use arena_core::{Action, GameKind, Outcome, Turn, View};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRoom {
    pub nickname: String,
    pub game: GameKind,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JoinRoom {
    pub nickname: String,
}

/// Keep the token private. It grants control of this seat, including after reconnecting.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Membership {
    pub code: String,
    pub player: usize,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayerView {
    pub nickname: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Snapshot {
    pub code: String,
    pub game: GameKind,
    pub players: Vec<PlayerView>,
    pub you: usize,
    pub revision: u64,
    pub turn: Option<Turn>,
    pub outcome: Option<Outcome>,
    pub view: Option<View>,
    pub legal_actions: Vec<Action>,
    pub replays: Vec<String>,
}

/// Send Authenticate as the first text frame within five seconds. Subsequent
/// commands must carry the most recent Snapshot revision.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    Authenticate { token: String },
    Start { revision: u64 },
    Move { revision: u64, action: Action },
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Snapshot { room: Box<Snapshot> },
    Error { error: String },
}
