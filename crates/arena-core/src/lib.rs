//! Deterministic game rules, reusable cards, validated sessions, and replays.

pub mod arena;
pub mod cards;
pub mod game;
pub mod games;

pub use arena::{Action, Config, GameKind, Match, Replay, View};
pub use game::{
    Error, Event, Game, GameEvent, Outcome, Player, Resolution, ResolutionRecord, Session, Turn,
};
