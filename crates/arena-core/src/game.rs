use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

pub type Player = usize;
pub type Seed = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Invalid player count")]
    PlayerCount,
    #[error("Not your turn")]
    NotYourTurn,
    #[error("Illegal move")]
    IllegalMove,
    #[error("Game is over")]
    Finished,
    #[error("Invalid replay")]
    Replay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Turn {
    pub player: Player,
    pub number: u64,
}

impl Turn {
    pub fn first() -> Self {
        Self {
            player: 0,
            number: 0,
        }
    }
    pub fn next(&mut self, count: usize) {
        self.player = (self.player + 1) % count;
        self.number += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Outcome {
    pub winners: Vec<Player>,
    pub scores: Option<Vec<i32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Played { player: Player },
    Captured { player: Player, count: usize },
    Burned,
    ColumnRemoved { player: Player, column: usize },
    Finished { outcome: Outcome },
}

/// A deterministic ruleset. No transport, identity, persistence, or clock dependencies.
/// `advance` receives a legal action; callers normally use `Session::act`.
pub trait Game: Clone {
    type Action: Clone + PartialEq + Serialize + DeserializeOwned;
    type View: Serialize;
    fn new(players: usize, seed: Seed) -> Result<Self, Error>;
    fn turn(&self) -> Turn;
    fn outcome(&self) -> Option<Outcome>;
    fn legal_actions(&self) -> Vec<Self::Action>;
    fn advance(&mut self, action: &Self::Action) -> Vec<Event>;
    fn view(&self, viewer: Player) -> Self::View;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Record<A> {
    pub player: Player,
    pub action: A,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session<G: Game> {
    pub(crate) state: G,
    pub(crate) seed: Seed,
    pub(crate) players: usize,
    #[serde(bound(deserialize = "G::Action: DeserializeOwned"))]
    pub(crate) records: Vec<Record<G::Action>>,
}

impl<G: Game> Session<G> {
    pub fn new(players: usize, seed: Seed) -> Result<Self, Error> {
        Ok(Self {
            state: G::new(players, seed)?,
            seed,
            players,
            records: Vec::new(),
        })
    }
    pub fn state(&self) -> &G {
        &self.state
    }
    pub fn records(&self) -> &[Record<G::Action>] {
        &self.records
    }
    pub fn act(&mut self, player: Player, action: G::Action) -> Result<&Record<G::Action>, Error> {
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
            events.push(Event::Finished { outcome });
        }
        self.records.push(Record {
            player,
            action,
            events,
        });
        Ok(self.records.last().expect("just appended"))
    }
}
