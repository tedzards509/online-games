//! Ruleset contracts, turn and outcome types, and validated action histories.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

/// A zero-based seat within a game, independent of names or transport identities.
pub type Player = usize;
/// The 32 bytes that determine a game's random setup and subsequent draws.
pub type Seed = [u8; 32];

/// Rejects persisted state from a different rules revision rather than reinterpreting it.
pub(crate) fn deserialize_rules_version<'de, D, const VERSION: u32>(
    deserializer: D,
) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u32::deserialize(deserializer)?;
    if version != VERSION {
        return Err(serde::de::Error::custom(format!(
            "unsupported rules version {version}; expected {VERSION}"
        )));
    }
    Ok(version)
}

/// Empty serializable configuration for games without optional rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {}

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

/// Eligible seats and a ruleset-managed turn counter, not a room revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Turn {
    /// The seats that may act next; this is not a queue or priority order.
    #[serde(alias = "player", deserialize_with = "deserialize_turn_players")]
    pub players: Vec<Player>,
    /// The number of turn advances since setup.
    pub number: u64,
}

impl Turn {
    /// Starts at seat zero with turn number zero.
    pub fn first() -> Self {
        Self {
            players: vec![0],
            number: 0,
        }
    }
    /// Moves a single eligible seat forward and increments the counter.
    /// Panics unless exactly one seat is eligible and `count` is positive.
    pub fn advance(&mut self, count: usize) {
        assert_eq!(
            self.players.len(),
            1,
            "advance requires exactly one eligible player"
        );
        assert!(count > 0, "player count must be positive");
        self.players[0] = (self.players[0] + 1) % count;
        self.number += 1;
    }
}

/// Reads both current eligibility lists and legacy persisted single-seat turns.
fn deserialize_turn_players<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Player>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Players {
        Many(Vec<Player>),
        One(Player),
    }
    Ok(match Players::deserialize(deserializer)? {
        Players::Many(players) => players,
        Players::One(player) => vec![player],
    })
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
    /// Serializable setup options owned by this ruleset.
    type Config: Clone + Default + PartialEq + Serialize + DeserializeOwned;
    /// A comparable command selected from the current legal actions.
    type Action: Clone + PartialEq + Serialize + DeserializeOwned;
    /// Rule effects recorded for each accepted action.
    type Event: GameEvent;
    /// A player's observation, with private information removed as needed.
    type View: Serialize;
    /// Creates a seeded initial state, rejecting unsupported player counts.
    fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        Self::with_config(players, seed, Self::Config::default())
    }
    /// Creates deterministic setup with explicit ruleset options.
    fn with_config(players: usize, seed: Seed, config: Self::Config) -> Result<Self, Error>;
    /// Returns eligible seats and the ruleset turn counter.
    fn turn(&self) -> Turn;
    /// Returns the final result, or `None` while play continues.
    fn outcome(&self) -> Option<Outcome>;
    /// Lists a seat's available actions; empty for ineligible seats or after completion.
    fn legal_actions(&self, player: Player) -> Vec<Self::Action>;
    /// Applies a legal action, updates turn order, and returns effects without a terminal event.
    fn advance(&mut self, player: Player, action: &Self::Action) -> Vec<Self::Event>;
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
    #[serde(default, bound(deserialize = "G::Config: DeserializeOwned"))]
    pub(crate) config: G::Config,
    #[serde(bound(deserialize = "G::Action: DeserializeOwned, G::Event: DeserializeOwned"))]
    pub(crate) records: Vec<Record<G::Action, G::Event>>,
}

impl<G: Game> Session<G> {
    /// Creates a game and an empty history from an explicit seed.
    pub fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        Self::with_config(players, seed, G::Config::default())
    }
    /// Creates a configured game and retains its options for persistence and replay.
    pub fn with_config(players: usize, seed: Seed, config: G::Config) -> Result<Self, Error> {
        Ok(Self {
            state: G::with_config(players, seed, config.clone())?,
            seed,
            players,
            config,
            records: Vec::new(),
        })
    }
    /// Borrows the full private state; use `Game::view` for client responses.
    pub fn state(&self) -> &G {
        &self.state
    }
    /// Borrows the immutable setup options.
    pub fn config(&self) -> &G::Config {
        &self.config
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
        if player >= self.players || !self.state.turn().players.contains(&player) {
            return Err(Error::NotYourTurn);
        }
        if !self.state.legal_actions(player).contains(&action) {
            return Err(Error::IllegalMove);
        }
        let mut events = self.state.advance(player, &action);
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
