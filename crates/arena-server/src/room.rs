use crate::{
    error::ApiError,
    protocol::{Membership, PlayerView, Snapshot},
};
use arena_core::{GameKind, Match};
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
    members: Vec<Member>,
    pub game: Option<Match>,
    pub revision: u64,
    pub replays: Vec<String>,
}

fn hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

impl Room {
    pub fn new(code: String, kind: GameKind) -> Self {
        Self {
            code,
            kind,
            members: vec![],
            game: None,
            revision: 0,
            replays: vec![],
        }
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
        self.game = Some(Match::new(self.kind, self.members.len(), rand::random())?);
        self.revision += 1;
        Ok(())
    }
    pub fn snapshot(&self, player: usize) -> Snapshot {
        Snapshot {
            code: self.code.clone(),
            game: self.kind,
            players: self
                .members
                .iter()
                .map(|m| PlayerView {
                    nickname: m.nickname.clone(),
                })
                .collect(),
            you: player,
            revision: self.revision,
            turn: self.game.as_ref().map(Match::turn),
            outcome: self.game.as_ref().and_then(Match::outcome),
            view: self.game.as_ref().map(|g| g.view(player)),
            legal_actions: self
                .game
                .as_ref()
                .map(|g| g.legal_actions(player))
                .unwrap_or_default(),
            replays: self.replays.clone(),
        }
    }
}
