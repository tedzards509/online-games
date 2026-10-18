//! Ruleset contracts, turn and outcome types, and validated action histories.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

/// A zero-based seat within a game, independent of names or transport identities.
pub type Player = usize;
/// The 32 bytes that determine a game's random setup and subsequent draws.
pub type Seed = [u8; 32];

/// A rejected setup, action, or replay.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The ruleset does not support the requested number of players.
    #[error("Invalid player count")]
    PlayerCount,
    /// The submitting seat is not active.
    #[error("Not your turn")]
    NotYourTurn,
    /// The action is absent from the current legal actions.
    #[error("Illegal move")]
    IllegalMove,
    /// The game has already ended.
    #[error("Game is over")]
    Finished,
    /// The replay version or regenerated history does not match.
    #[error("Invalid replay")]
    Replay,
}

/// The active seat and a ruleset-managed turn counter, not a room revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Turn {
    /// The seat that may act next.
    pub player: Player,
    /// The number of turn advances since setup.
    pub number: u64,
}

impl Turn {
    /// Starts at seat zero with turn number zero.
    pub fn first() -> Self {
        Self {
            player: 0,
            number: 0,
        }
    }
    /// Moves to the next seat and increments the counter; `count` must be positive.
    pub fn next(&mut self, count: usize) {
        self.player = (self.player + 1) % count;
        self.number += 1;
    }
}

/// A finished result with optional scores in seat order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Outcome {
    /// Winning seats; empty for a draw, or multiple seats for a shared win.
    pub winners: Vec<Player>,
    /// Optional final scores; each ruleset determines how they rank.
    pub scores: Option<Vec<i32>>,
}

/// Common rule effects that the included games can record without exposing cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A player performed an accepted action.
    Played { player: Player },
    /// A player captured pieces from the board.
    Captured { player: Player, count: usize },
    /// The current play pile was removed from the game.
    Burned,
    /// A matching column was removed from a player's layout.
    ColumnRemoved { player: Player, column: usize },
    /// The session reached its final outcome.
    Finished { outcome: Outcome },
}

/// An event type that can represent the session's final result.
pub trait GameEvent: Clone + PartialEq + Serialize + DeserializeOwned {
    /// Creates the terminal event appended by `Session::act`.
    fn finished(outcome: Outcome) -> Self;
}

impl GameEvent for Event {
    fn finished(outcome: Outcome) -> Self {
        Self::Finished { outcome }
    }
}

/// A deterministic ruleset. No transport, identity, persistence, or clock dependencies.
/// `advance` receives a legal action; callers normally use `Session::act`.
pub trait Game: Clone {
    /// A comparable command selected from the current legal actions.
    type Action: Clone + PartialEq + Serialize + DeserializeOwned;
    /// Rule effects recorded for each accepted action.
    type Event: GameEvent;
    /// A player's observation, with private information removed as needed.
    type View: Serialize;
    /// Creates a seeded initial state, rejecting unsupported player counts.
    fn new(players: usize, seed: Seed) -> Result<Self, Error>;
    /// Returns the active seat and ruleset turn counter.
    fn turn(&self) -> Turn;
    /// Returns the final result, or `None` while play continues.
    fn outcome(&self) -> Option<Outcome>;
    /// Lists the active player's available actions; empty after completion.
    fn legal_actions(&self) -> Vec<Self::Action>;
    /// Applies a legal action, updates turn order, and returns effects without a terminal event.
    fn advance(&mut self, action: &Self::Action) -> Vec<Self::Event>;
    /// Projects state for one viewer without leaking other players' private information.
    fn view(&self, viewer: Player) -> Self::View;
}

/// One accepted action and the ordered effects it produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Record<A, E> {
    /// The seat that submitted the action.
    pub player: Player,
    /// The command accepted by the ruleset.
    pub action: A,
    /// Rule effects, followed by a terminal event when the game ends.
    pub events: Vec<E>,
}

/// A private ruleset state with its seed, player count, and accepted-action log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session<G: Game> {
    pub(crate) state: G,
    pub(crate) seed: Seed,
    pub(crate) players: usize,
    #[serde(bound(deserialize = "G::Action: DeserializeOwned, G::Event: DeserializeOwned"))]
    pub(crate) records: Vec<Record<G::Action, G::Event>>,
}

impl<G: Game> Session<G> {
    /// Creates a game and an empty history from an explicit seed.
    pub fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        Ok(Self {
            state: G::new(players, seed)?,
            seed,
            players,
            records: Vec::new(),
        })
    }
    /// Borrows the full private state; use `Game::view` for client responses.
    pub fn state(&self) -> &G {
        &self.state
    }
    /// Borrows the accepted-action history in execution order.
    pub fn records(&self) -> &[Record<G::Action, G::Event>] {
        &self.records
    }
    /// Rejects finished, out-of-turn, or illegal actions before changing state or history.
    pub fn act(
        &mut self,
        player: Player,
        action: G::Action,
    ) -> Result<&Record<G::Action, G::Event>, Error> {
        if self.state.outcome().is_some() {
            return Err(Error::Finished);
        }
        if self.state.turn().player != player {
            return Err(Error::NotYourTurn);
        }
        if !self.state.legal_actions().contains(&action) {
            return Err(Error::IllegalMove);
        }
        let mut events = self.state.advance(&action);
        if let Some(outcome) = self.state.outcome() {
            events.push(G::Event::finished(outcome));
        }
        self.records.push(Record {
            player,
            action,
            events,
        });
        Ok(self.records.last().expect("just appended"))
    }
}
