use crate::{
    game::{Error, Event, Game, Outcome, Player, Record, Seed, Session, Turn},
    games::{crazy_eights, mancala, shithead, skyjo},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameKind {
    Mancala,
    CrazyEights,
    Shithead,
    Skyjo,
}

impl GameKind {
    pub fn max_players(self) -> usize {
        match self {
            Self::Mancala => 2,
            Self::CrazyEights => 6,
            Self::Shithead => 5,
            Self::Skyjo => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "game", content = "action", rename_all = "snake_case")]
pub enum Action {
    Mancala(mancala::Action),
    CrazyEights(crazy_eights::Action),
    Shithead(shithead::Action),
    Skyjo(skyjo::Action),
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "game", content = "state", rename_all = "snake_case")]
pub enum View {
    Mancala(mancala::View),
    CrazyEights(crazy_eights::View),
    Shithead(shithead::View),
    Skyjo(skyjo::View),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Match {
    Mancala(Session<mancala::Mancala>),
    CrazyEights(Session<crazy_eights::CrazyEights>),
    Shithead(Session<shithead::Shithead>),
    Skyjo(Session<skyjo::Skyjo>),
}

macro_rules! dispatch {
    ($this:expr, $session:ident => $body:expr) => {
        match $this {
            Match::Mancala($session) => $body,
            Match::CrazyEights($session) => $body,
            Match::Shithead($session) => $body,
            Match::Skyjo($session) => $body,
        }
    };
}

impl Match {
    pub fn new(kind: GameKind, players: usize, seed: Seed) -> Result<Self, Error> {
        Ok(match kind {
            GameKind::Mancala => Self::Mancala(Session::new(players, seed)?),
            GameKind::CrazyEights => Self::CrazyEights(Session::new(players, seed)?),
            GameKind::Shithead => Self::Shithead(Session::new(players, seed)?),
            GameKind::Skyjo => Self::Skyjo(Session::new(players, seed)?),
        })
    }
    pub fn turn(&self) -> Turn {
        dispatch!(self, s => s.state().turn())
    }
    pub fn outcome(&self) -> Option<Outcome> {
        dispatch!(self, s => s.state().outcome())
    }
    pub fn view(&self, player: Player) -> View {
        match self {
            Self::Mancala(s) => View::Mancala(s.state().view(player)),
            Self::CrazyEights(s) => View::CrazyEights(s.state().view(player)),
            Self::Shithead(s) => View::Shithead(s.state().view(player)),
            Self::Skyjo(s) => View::Skyjo(s.state().view(player)),
        }
    }
    pub fn legal_actions(&self, player: Player) -> Vec<Action> {
        if self.turn().player != player {
            return vec![];
        }
        match self {
            Self::Mancala(s) => s
                .state()
                .legal_actions()
                .into_iter()
                .map(Action::Mancala)
                .collect(),
            Self::CrazyEights(s) => s
                .state()
                .legal_actions()
                .into_iter()
                .map(Action::CrazyEights)
                .collect(),
            Self::Shithead(s) => s
                .state()
                .legal_actions()
                .into_iter()
                .map(Action::Shithead)
                .collect(),
            Self::Skyjo(s) => s
                .state()
                .legal_actions()
                .into_iter()
                .map(Action::Skyjo)
                .collect(),
        }
    }
    pub fn act(&mut self, player: Player, action: Action) -> Result<(), Error> {
        match (self, action) {
            (Self::Mancala(s), Action::Mancala(a)) => {
                s.act(player, a)?;
            }
            (Self::CrazyEights(s), Action::CrazyEights(a)) => {
                s.act(player, a)?;
            }
            (Self::Shithead(s), Action::Shithead(a)) => {
                s.act(player, a)?;
            }
            (Self::Skyjo(s), Action::Skyjo(a)) => {
                s.act(player, a)?;
            }
            _ => return Err(Error::IllegalMove),
        }
        Ok(())
    }
    pub fn replay(&self) -> Replay {
        let records = match self {
            Self::Mancala(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::Mancala))
                .collect(),
            Self::CrazyEights(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::CrazyEights))
                .collect(),
            Self::Shithead(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::Shithead))
                .collect(),
            Self::Skyjo(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::Skyjo))
                .collect(),
        };
        let game = match self {
            Self::Mancala(_) => GameKind::Mancala,
            Self::CrazyEights(_) => GameKind::CrazyEights,
            Self::Shithead(_) => GameKind::Shithead,
            Self::Skyjo(_) => GameKind::Skyjo,
        };
        Replay {
            version: 1,
            game,
            players: dispatch!(self, s => s.players),
            seed: dispatch!(self, s => s.seed),
            records,
        }
    }
}

impl<A: Clone, E: Clone> Record<A, E> {
    fn map_action<B>(&self, f: impl FnOnce(A) -> B) -> Record<B, E> {
        Record {
            player: self.player,
            action: f(self.action.clone()),
            events: self.events.clone(),
        }
    }
}

/// Includes hidden information; only share after the game has ended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Replay {
    pub version: u32,
    pub game: GameKind,
    pub players: usize,
    #[schema(value_type = Vec<u8>, min_items = 32, max_items = 32)]
    pub seed: Seed,
    pub records: Vec<Record<Action, Event>>,
}

impl Replay {
    pub fn simulate(&self) -> Result<Match, Error> {
        if self.version != 1 {
            return Err(Error::Replay);
        }
        let mut game = Match::new(self.game, self.players, self.seed)?;
        for record in &self.records {
            game.act(record.player, record.action.clone())?;
        }
        if game.replay() != *self {
            return Err(Error::Replay);
        }
        Ok(game)
    }
}
