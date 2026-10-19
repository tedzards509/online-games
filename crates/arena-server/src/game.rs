//! Game-instance routing, member-to-player mapping, and host-only game controls.

use crate::{
    error::ApiError,
    protocol::{GameMessage, GameSnapshot, MemberView, ReplaySummary},
};
use arena_core::{
    Action, Match,
    games::{tempel_des_schreckens, the_crew},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoomGame {
    pub id: String,
    pub started_at: u64,
    pub players: Vec<MemberView>,
    pub session: Match,
}

fn host_action(action: &Action) -> bool {
    matches!(
        action,
        Action::TheCrew(the_crew::Action::Finish { .. })
            | Action::TempelDesSchreckens(tempel_des_schreckens::Action::ChooseKey { .. })
    )
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
                if host_action(&action) {
                    return Err(ApiError::BadRequest("Use the host game control".into()));
                }
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
                if !matches!(self.session, Match::TheCrew(_)) {
                    return Err(ApiError::BadRequest(
                        "This game determines its own result".into(),
                    ));
                }
                // Player zero records external decisions deterministically; room
                // membership, not this player index, determines host authority.
                self.session
                    .act(0, Action::TheCrew(the_crew::Action::Finish { won }))?;
            }
            GameMessage::ChooseKey { player } => {
                if !matches!(self.session, Match::TempelDesSchreckens(_)) {
                    return Err(ApiError::BadRequest("This game has no key".into()));
                }
                self.session.act(
                    0,
                    Action::TempelDesSchreckens(tempel_des_schreckens::Action::ChooseKey {
                        player,
                    }),
                )?;
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
        let legal = |player| {
            self.session
                .legal_actions(player)
                .into_iter()
                .filter(|a| !host_action(a))
                .collect::<Vec<_>>()
        };
        let mut turn = self.session.turn();
        turn.players.retain(|&p| !legal(p).is_empty());
        let controls = self.session.legal_actions(0);
        GameSnapshot {
            id: self.id.clone(),
            config: self.session.config(),
            players: self.players.clone(),
            you,
            can_finish: is_host
                && controls
                    .iter()
                    .any(|a| matches!(a, Action::TheCrew(the_crew::Action::Finish { .. }))),
            can_choose_key: is_host
                && controls.iter().any(|a| {
                    matches!(
                        a,
                        Action::TempelDesSchreckens(
                            tempel_des_schreckens::Action::ChooseKey { .. }
                        )
                    )
                }),
            turn,
            outcome: self.session.outcome(),
            view: you.map(|p| self.session.view(p)),
            legal_actions: you.map(legal).unwrap_or_default(),
        }
    }
}
