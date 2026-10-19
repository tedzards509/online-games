//! Game-instance routing, member-to-player mapping, and host-only game controls.

use crate::{
    error::ApiError,
    protocol::{GameMessage, GameSnapshot, MemberView, ReplaySummary},
};
use arena_core::{Match, Resolution};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoomGame {
    pub id: String,
    pub started_at: u64,
    pub players: Vec<MemberView>,
    pub session: Match,
}

impl RoomGame {
    pub fn act(
        &mut self,
        member: usize,
        is_host: bool,
        message: GameMessage,
    ) -> Result<(), ApiError> {
        if !matches!(message, GameMessage::Move { .. }) && !is_host {
            return Err(ApiError::BadRequest("Only the host may do this".into()));
        }
        match message {
            GameMessage::Move { action } => {
                let player = self
                    .players
                    .iter()
                    .position(|p| p.id == member)
                    .ok_or_else(|| {
                        ApiError::BadRequest("Member is not a player in this game".into())
                    })?;
                self.session.act(player, action)?;
            }
            GameMessage::Finish { won } => {
                self.session.resolve(Resolution::SharedResult { won })?;
            }
        }
        Ok(())
    }

    pub fn replay_summary(&self) -> ReplaySummary {
        ReplaySummary {
            id: self.id.clone(),
            config: self.session.config(),
            players: self.players.clone(),
        }
    }

    pub fn snapshot(&self, member: usize, is_host: bool) -> GameSnapshot {
        let you = self.players.iter().position(|p| p.id == member);
        GameSnapshot {
            id: self.id.clone(),
            config: self.session.config(),
            players: self.players.clone(),
            you,
            can_finish: is_host
                && self
                    .session
                    .resolutions()
                    .contains(&Resolution::SharedResult { won: true }),
            turn: self.session.turn(),
            outcome: self.session.outcome(),
            view: you.map(|p| self.session.view(p)),
            legal_actions: you
                .map(|p| self.session.legal_actions(p))
                .unwrap_or_default(),
        }
    }
}
