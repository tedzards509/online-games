//! Runtime game dispatch, transport-facing actions and views, and replay validation.

use crate::{
    game::{Error, Event, Game, Outcome, Player, Record, Seed, Session, Turn},
    games::{crazy_eights, mancala, shithead, skyjo, tempel_des_schreckens, the_crew, tic_tac_toe},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A registered ruleset and its serialized game identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameKind {
    TempelDesSchreckens,
    TheCrew,
    Mancala,
    CrazyEights,
    Shithead,
    Skyjo,
    TicTacToe,
}

impl GameKind {
    /// Returns the currently supported replay rules revision for this game.
    pub fn replay_version(self) -> u32 {
        match self {
            Self::CrazyEights | Self::Mancala | Self::Shithead | Self::Skyjo => 4,
            _ => 1,
        }
    }
    /// Returns the ruleset's default setup options.
    pub fn default_config(self) -> Config {
        match self {
            Self::TempelDesSchreckens => Config::TempelDesSchreckens(Default::default()),
            Self::TheCrew => Config::TheCrew(Default::default()),
            Self::Mancala => Config::Mancala(Default::default()),
            Self::CrazyEights => Config::CrazyEights(Default::default()),
            Self::Shithead => Config::Shithead(Default::default()),
            Self::Skyjo => Config::Skyjo(Default::default()),
            Self::TicTacToe => Config::TicTacToe(Default::default()),
        }
    }
    /// Returns the largest supported room size for this ruleset.
    pub fn max_players(self) -> usize {
        match self {
            Self::TempelDesSchreckens => 10,
            Self::TheCrew => 5,
            Self::Mancala => 2,
            Self::CrazyEights => 6,
            Self::Shithead => 5,
            Self::Skyjo => 8,
            Self::TicTacToe => 2,
        }
    }
}

/// Setup options tagged with the ruleset that defines their `GameConfig` struct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(
    tag = "game",
    content = "rules",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Config {
    TempelDesSchreckens(crate::game::GameConfig),
    TheCrew(crate::game::GameConfig),
    Mancala(crate::game::GameConfig),
    CrazyEights(crazy_eights::GameConfig),
    Shithead(shithead::GameConfig),
    Skyjo(crate::game::GameConfig),
    TicTacToe(crate::game::GameConfig),
}

impl Config {
    /// Returns the ruleset these options belong to.
    pub fn kind(&self) -> GameKind {
        match self {
            Self::TempelDesSchreckens(_) => GameKind::TempelDesSchreckens,
            Self::TheCrew(_) => GameKind::TheCrew,
            Self::Mancala(_) => GameKind::Mancala,
            Self::CrazyEights(_) => GameKind::CrazyEights,
            Self::Shithead(_) => GameKind::Shithead,
            Self::Skyjo(_) => GameKind::Skyjo,
            Self::TicTacToe(_) => GameKind::TicTacToe,
        }
    }
}

/// A command tagged with the ruleset that understands it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "game", content = "action", rename_all = "snake_case")]
pub enum Action {
    TempelDesSchreckens(tempel_des_schreckens::Action),
    TheCrew(the_crew::Action),
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
    TempelDesSchreckens(tempel_des_schreckens::View),
    TheCrew(the_crew::View),
    Mancala(mancala::View),
    CrazyEights(crazy_eights::View),
    Shithead(shithead::View),
    Skyjo(skyjo::View),
    TicTacToe(tic_tac_toe::View),
}

/// A private session selected at runtime; never send it as a live response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Match {
    TempelDesSchreckens(Session<tempel_des_schreckens::TempelDesSchreckens>),
    TheCrew(Session<the_crew::TheCrew>),
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
            Match::TempelDesSchreckens($session) => $body,
            Match::TheCrew($session) => $body,
            Match::Mancala($session) => $body,
            Match::CrazyEights($session) => $body,
            Match::Shithead($session) => $body,
            Match::Skyjo($session) => $body,
            Match::TicTacToe($session) => $body,
        }
    };
}

impl Match {
    /// Counts accepted actions in the current game, excluding previous rematches.
    pub fn action_count(&self) -> usize {
        dispatch!(self, s => s.records.len())
    }

    /// Creates the selected ruleset with the supplied seats and seed.
    pub fn new(kind: GameKind, players: usize, seed: Seed) -> Result<Self, Error> {
        Self::with_config(kind.default_config(), players, seed)
    }
    /// Creates a match with explicit game-owned setup options.
    pub fn with_config(config: Config, players: usize, seed: Seed) -> Result<Self, Error> {
        Ok(match config {
            Config::TempelDesSchreckens(c) => {
                Self::TempelDesSchreckens(Session::with_config(players, seed, c)?)
            }
            Config::TheCrew(c) => Self::TheCrew(Session::with_config(players, seed, c)?),
            Config::Mancala(c) => Self::Mancala(Session::with_config(players, seed, c)?),
            Config::CrazyEights(c) => Self::CrazyEights(Session::with_config(players, seed, c)?),
            Config::Shithead(c) => Self::Shithead(Session::with_config(players, seed, c)?),
            Config::Skyjo(c) => Self::Skyjo(Session::with_config(players, seed, c)?),
            Config::TicTacToe(c) => Self::TicTacToe(Session::with_config(players, seed, c)?),
        })
    }
    /// Returns public setup options without exposing private state.
    pub fn config(&self) -> Config {
        match self {
            Self::TempelDesSchreckens(s) => Config::TempelDesSchreckens(s.config().clone()),
            Self::TheCrew(s) => Config::TheCrew(s.config().clone()),
            Self::Mancala(s) => Config::Mancala(s.config().clone()),
            Self::CrazyEights(s) => Config::CrazyEights(s.config().clone()),
            Self::Shithead(s) => Config::Shithead(s.config().clone()),
            Self::Skyjo(s) => Config::Skyjo(s.config().clone()),
            Self::TicTacToe(s) => Config::TicTacToe(s.config().clone()),
        }
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
            Self::TempelDesSchreckens(s) => View::TempelDesSchreckens(s.state().view(player)),
            Self::TheCrew(s) => View::TheCrew(s.state().view(player)),
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
            Self::TempelDesSchreckens(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::TempelDesSchreckens)
                .collect(),
            Self::TheCrew(s) => s
                .state()
                .legal_actions(player)
                .into_iter()
                .map(Action::TheCrew)
                .collect(),
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
            (Self::TempelDesSchreckens(s), Action::TempelDesSchreckens(a)) => {
                s.act(player, a)?;
            }
            (Self::TheCrew(s), Action::TheCrew(a)) => {
                s.act(player, a)?;
            }
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
            Self::TempelDesSchreckens(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::TempelDesSchreckens))
                .collect(),
            Self::TheCrew(s) => s
                .records
                .iter()
                .map(|r| r.map_action(Action::TheCrew))
                .collect(),
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
            Self::TempelDesSchreckens(_) => GameKind::TempelDesSchreckens,
            Self::TheCrew(_) => GameKind::TheCrew,
            Self::Mancala(_) => GameKind::Mancala,
            Self::CrazyEights(_) => GameKind::CrazyEights,
            Self::Shithead(_) => GameKind::Shithead,
            Self::Skyjo(_) => GameKind::Skyjo,
            Self::TicTacToe(_) => GameKind::TicTacToe,
        };
        let config = self.config();
        let version = game.replay_version();
        Replay {
            version,
            config: (version > 1).then_some(config),
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
    /// Rules revision; must match the selected game's supported version.
    pub version: u32,
    /// The ruleset needed to interpret the history.
    pub game: GameKind,
    /// Explicit setup options; absent from version-1 files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Config>,
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
        if self.version != self.game.replay_version() {
            return Err(Error::Replay);
        }
        let config = match (self.version, &self.config) {
            (1, None) => self.game.default_config(),
            (4, Some(config)) if config.kind() == self.game => config.clone(),
            _ => return Err(Error::Replay),
        };
        let mut game = Match::with_config(config, self.players, self.seed)?;
        for record in &self.records {
            game.act(record.player, record.action.clone())?;
        }
        if game.replay() != *self {
            return Err(Error::Replay);
        }
        Ok(game)
    }
}
