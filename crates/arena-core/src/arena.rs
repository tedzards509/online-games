//! Runtime game dispatch, transport-facing actions and views, and replay validation.

use crate::{
    game::{Error, Event, Game, Outcome, Player, Record, Seed, Session, Turn},
    games::{crazy_eights, mancala, shithead, skyjo, tic_tac_toe},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A registered ruleset and its serialized game identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameKind {
    Mancala,
    CrazyEights,
    Shithead,
    Skyjo,
    TicTacToe,
}

impl GameKind {
    /// Returns the largest supported room size for this ruleset.
    pub fn max_players(self) -> usize {
        match self {
            Self::Mancala => 2,
            Self::CrazyEights => 6,
            Self::Shithead => 5,
            Self::Skyjo => 8,
            Self::TicTacToe => 2,
        }
    }
}

/// A command tagged with the ruleset that understands it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "game", content = "action", rename_all = "snake_case")]
pub enum Action {
    Mancala(mancala::Action),
    CrazyEights(crazy_eights::Action),
    Shithead(shithead::Action),
    Skyjo(skyjo::Action),
    TicTacToe(tic_tac_toe::Action),
}

/// A player's observation tagged with its ruleset.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "game", content = "state", rename_all = "snake_case")]
pub enum View {
    Mancala(mancala::View),
    CrazyEights(crazy_eights::View),
    Shithead(shithead::View),
    Skyjo(skyjo::View),
    TicTacToe(tic_tac_toe::View),
}

/// A private session selected at runtime; never send it as a live response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Match {
    Mancala(Session<mancala::Mancala>),
    CrazyEights(Session<crazy_eights::CrazyEights>),
    Shithead(Session<shithead::Shithead>),
    Skyjo(Session<skyjo::Skyjo>),
    TicTacToe(Session<tic_tac_toe::TicTacToe>),
}

/// Applies a shared session expression to any registered ruleset.
macro_rules! dispatch {
    ($this:expr, $session:ident => $body:expr) => {
        match $this {
            Match::Mancala($session) => $body,
            Match::CrazyEights($session) => $body,
            Match::Shithead($session) => $body,
            Match::Skyjo($session) => $body,
            Match::TicTacToe($session) => $body,
        }
    };
}

impl Match {
    /// Creates the selected ruleset with the supplied seats and seed.
    pub fn new(kind: GameKind, players: usize, seed: Seed) -> Result<Self, Error> {
        Ok(match kind {
            GameKind::Mancala => Self::Mancala(Session::new(players, seed)?),
            GameKind::CrazyEights => Self::CrazyEights(Session::new(players, seed)?),
            GameKind::Shithead => Self::Shithead(Session::new(players, seed)?),
            GameKind::Skyjo => Self::Skyjo(Session::new(players, seed)?),
            GameKind::TicTacToe => Self::TicTacToe(Session::new(players, seed)?),
        })
    }
    /// Returns the ruleset's eligible seats and turn counter.
    pub fn turn(&self) -> Turn {
        dispatch!(self, s => s.state().turn())
    }
    /// Returns the final result, if the match has ended.
    pub fn outcome(&self) -> Option<Outcome> {
        dispatch!(self, s => s.state().outcome())
    }
    /// Builds a personalized observation rather than exposing private state.
    pub fn view(&self, player: Player) -> View {
        match self {
            Self::Mancala(s) => View::Mancala(s.state().view(player)),
            Self::CrazyEights(s) => View::CrazyEights(s.state().view(player)),
            Self::Shithead(s) => View::Shithead(s.state().view(player)),
            Self::Skyjo(s) => View::Skyjo(s.state().view(player)),
            Self::TicTacToe(s) => View::TicTacToe(s.state().view(player)),
        }
    }
    /// Lists a seat's legal commands; ineligible seats receive an empty list.
    pub fn legal_actions(&self, player: Player) -> Vec<Action> {
        if !self.turn().players.contains(&player) {
            return vec![];
        }
        match self {
            Self::Mancala(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::Mancala)
                .collect(),
            Self::CrazyEights(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::CrazyEights)
                .collect(),
            Self::Shithead(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::Shithead)
                .collect(),
            Self::Skyjo(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::Skyjo)
                .collect(),
            Self::TicTacToe(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::TicTacToe)
                .collect(),
        }
    }
    /// Validates and records a command, rejecting mismatched game tags.
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
            (Self::TicTacToe(s), Action::TicTacToe(a)) => {
                s.act(player, a)?;
            }
            _ => return Err(Error::IllegalMove),
        }
        Ok(())
    }
    /// Exports the seed and full history, including secrets; share only after completion.
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
            Self::TicTacToe(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::TicTacToe))
                .collect(),
        };
        let game = match self {
            Self::Mancala(_) => GameKind::Mancala,
            Self::CrazyEights(_) => GameKind::CrazyEights,
            Self::Shithead(_) => GameKind::Shithead,
            Self::Skyjo(_) => GameKind::Skyjo,
            Self::TicTacToe(_) => GameKind::TicTacToe,
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
    /// Converts a command type while preserving its submitting seat and effects.
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
    /// The rules and randomization format; currently only version 1 is supported.
    pub version: u32,
    /// The ruleset needed to interpret the history.
    pub game: GameKind,
    /// The number of seats present at setup.
    pub players: usize,
    /// The original seed, sufficient to reconstruct hidden card order.
    #[schema(value_type = Vec<u8>, min_items = 32, max_items = 32)]
    pub seed: Seed,
    /// Accepted commands and their expected effects in execution order.
    pub records: Vec<Record<Action, Event>>,
}

impl Replay {
    /// Rebuilds the match, rejecting unsupported versions, invalid actions, or altered effects.
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
