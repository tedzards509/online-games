use crate::{
    error::ApiError,
    protocol::{Membership, PlayerView, Snapshot},
};
use arena_core::{Action, Config, GameKind, Match, games::the_crew};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Member {
    nickname: String,
    token_hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Room {
    pub code: String,
    pub kind: GameKind,
    #[serde(default)]
    config: Option<Config>,
    members: Vec<Member>,
    pub game: Option<Match>,
    pub revision: u64,
    pub replays: Vec<String>,
}

fn hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

impl Room {
    pub fn new(code: String, config: Config) -> Self {
        Self {
            code,
            kind: config.kind(),
            config: Some(config),
            members: vec![],
            game: None,
            revision: 0,
            replays: vec![],
        }
    }
    fn config(&self) -> Config {
        self.config
            .clone()
            .unwrap_or_else(|| self.kind.default_config())
    }
    pub fn join(&mut self, nickname: &str) -> Result<Membership, ApiError> {
        let nickname = nickname.trim();
        if nickname.is_empty()
            || nickname.chars().count() > 24
            || nickname.chars().any(char::is_control)
        {
            return Err(ApiError::BadRequest(
                "Use a nickname of 1–24 characters".into(),
            ));
        }
        if self.game.is_some() {
            return Err(ApiError::BadRequest("Game already started".into()));
        }
        if self.members.len() >= self.kind.max_players() {
            return Err(ApiError::BadRequest("Room is full".into()));
        }
        if self
            .members
            .iter()
            .any(|m| m.nickname.to_lowercase() == nickname.to_lowercase())
        {
            return Err(ApiError::BadRequest("Nickname already taken".into()));
        }
        let token = Uuid::new_v4().simple().to_string();
        let player = self.members.len();
        self.members.push(Member {
            nickname: nickname.into(),
            token_hash: hash(&token),
        });
        self.revision += 1;
        Ok(Membership {
            code: self.code.clone(),
            player,
            token,
        })
    }
    pub fn authenticate(&self, token: &str) -> Result<usize, ApiError> {
        let digest = hash(token);
        self.members
            .iter()
            .position(|m| m.token_hash == digest)
            .ok_or(ApiError::Unauthorized)
    }
    pub fn start(&mut self, player: usize) -> Result<(), ApiError> {
        if player != 0 {
            return Err(ApiError::BadRequest("Only the host can start".into()));
        }
        if self.game.as_ref().is_some_and(|g| g.outcome().is_none()) {
            return Err(ApiError::BadRequest("Game already started".into()));
        }
        self.game = Some(Match::with_config(
            self.config(),
            self.members.len(),
            rand::random(),
        )?);
        self.revision += 1;
        Ok(())
    }
    pub fn snapshot(&self, player: usize) -> Snapshot {
        Snapshot {
            code: self.code.clone(),
            game: self.kind,
            config: self.config(),
            players: self
                .members
                .iter()
                .map(|m| PlayerView {
                    nickname: m.nickname.clone(),
                })
                .collect(),
            you: player,
            revision: self.revision,
            can_finish: player == 0
                && self.kind == GameKind::TheCrew
                && self.game.as_ref().is_some_and(|g| g.outcome().is_none()),
            turn: self.game.as_ref().map(|game| {
                let mut turn = game.turn();
                turn.players
                    .retain(|&seat| !self.legal_actions(seat).is_empty());
                turn
            }),
            outcome: self.game.as_ref().and_then(Match::outcome),
            view: self.game.as_ref().map(|g| g.view(player)),
            legal_actions: self.legal_actions(player),
            replays: self.replays.clone(),
        }
    }

    fn legal_actions(&self, player: usize) -> Vec<Action> {
        self.game
            .as_ref()
            .map(|g| {
                g.legal_actions(player)
                    .into_iter()
                    .filter(|a| !matches!(a, Action::TheCrew(the_crew::Action::Finish { .. })))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn finish(&mut self, player: usize, won: bool) -> Result<(), ApiError> {
        if player != 0 {
            return Err(ApiError::BadRequest("Only the host can finish".into()));
        }
        let game = self
            .game
            .as_mut()
            .ok_or_else(|| ApiError::BadRequest("Game has not started".into()))?;
        if !matches!(game, Match::TheCrew(_)) {
            return Err(ApiError::BadRequest(
                "This game determines its own result".into(),
            ));
        }
        game.act(player, Action::TheCrew(the_crew::Action::Finish { won }))?;
        self.revision += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_games_do_not_offer_or_accept_manual_room_results() {
        let mut room = Room::new("ABC234".into(), GameKind::Mancala.default_config());
        room.join("Alice").unwrap();
        room.join("Bob").unwrap();
        room.start(0).unwrap();
        assert!(!room.snapshot(0).can_finish);
        let before = serde_json::to_value(&room).unwrap();
        assert!(room.finish(0, true).is_err());
        assert_eq!(serde_json::to_value(&room).unwrap(), before);
    }

    #[test]
    fn legacy_rooms_without_options_use_original_defaults() {
        let mut room = Room::new("ABC234".into(), GameKind::CrazyEights.default_config());
        room.join("Alice").unwrap();
        room.join("Bob").unwrap();
        let mut saved = serde_json::to_value(room).unwrap();
        saved.as_object_mut().unwrap().remove("config");
        let mut room: Room = serde_json::from_value(saved).unwrap();
        assert_eq!(
            room.snapshot(1).config,
            GameKind::CrazyEights.default_config()
        );
        room.start(0).unwrap();
        assert_eq!(
            room.game.unwrap().config(),
            GameKind::CrazyEights.default_config()
        );
    }
}
