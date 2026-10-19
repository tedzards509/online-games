use crate::{
    error::ApiError,
    game::RoomGame,
    protocol::{GameMessage, MemberView, Membership, ReplaySummary, Snapshot},
};
use arena_core::{Config, Match};
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
    pub host: usize,
    config: Option<Config>,
    members: Vec<Member>,
    seating: Vec<usize>,
    pub game: Option<RoomGame>,
    pub revision: u64,
    pub replays: Vec<ReplaySummary>,
}

fn hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

impl Room {
    pub fn new(code: String) -> Self {
        Self {
            code,
            host: 0,
            config: None,
            members: vec![],
            seating: vec![],
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
        if self
            .members
            .iter()
            .any(|m| m.nickname.to_lowercase() == nickname.to_lowercase())
        {
            return Err(ApiError::BadRequest("Nickname already taken".into()));
        }
        let token = Uuid::new_v4().simple().to_string();
        let member = self.members.len();
        self.members.push(Member {
            nickname: nickname.into(),
            token_hash: hash(&token),
        });
        self.revision += 1;
        Ok(Membership {
            code: self.code.clone(),
            member,
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

    fn require_host(&self, member: usize) -> Result<(), ApiError> {
        if member != self.host {
            return Err(ApiError::BadRequest("Only the host may do this".into()));
        }
        Ok(())
    }

    fn require_setup(&self, member: usize) -> Result<(), ApiError> {
        self.require_host(member)?;
        if self
            .game
            .as_ref()
            .is_some_and(|g| g.session.outcome().is_none())
        {
            return Err(ApiError::BadRequest("Game is active".into()));
        }
        Ok(())
    }

    pub fn configure(&mut self, member: usize, config: Option<Config>) -> Result<(), ApiError> {
        self.require_setup(member)?;
        self.config = config;
        self.revision += 1;
        Ok(())
    }

    pub fn seat(&mut self, member: usize, members: Vec<usize>) -> Result<(), ApiError> {
        self.require_setup(member)?;
        let mut seen = std::collections::HashSet::new();
        if members
            .iter()
            .any(|&m| m >= self.members.len() || !seen.insert(m))
        {
            return Err(ApiError::BadRequest(
                "Seats must contain distinct room members".into(),
            ));
        }
        self.seating = members;
        self.revision += 1;
        Ok(())
    }

    pub fn start(&mut self, member: usize) -> Result<(), ApiError> {
        self.require_setup(member)?;
        let config = self
            .config
            .clone()
            .ok_or_else(|| ApiError::BadRequest("Choose a game first".into()))?;
        let session = Match::with_config(config, self.seating.len(), rand::random())?;
        self.revision += 1;
        self.game = Some(RoomGame {
            id: Uuid::new_v4().to_string(),
            started_at: self.revision,
            players: self.seating.iter().map(|&m| self.member_view(m)).collect(),
            session,
        });
        Ok(())
    }

    pub fn abort(&mut self, member: usize, game_id: &str) -> Result<(), ApiError> {
        self.require_host(member)?;
        self.check_game(game_id)?;
        self.game = None;
        self.revision += 1;
        Ok(())
    }

    pub fn check_game(&self, id: &str) -> Result<&RoomGame, ApiError> {
        self.game
            .as_ref()
            .filter(|g| g.id == id)
            .ok_or(ApiError::Conflict)
    }

    pub fn act(&mut self, member: usize, message: GameMessage) -> Result<(), ApiError> {
        let game = self
            .game
            .as_mut()
            .ok_or_else(|| ApiError::BadRequest("Game has not started".into()))?;
        game.act(member, member == self.host, message)?;
        self.revision += 1;
        Ok(())
    }

    fn member_view(&self, id: usize) -> MemberView {
        MemberView {
            id,
            nickname: self.members[id].nickname.clone(),
        }
    }

    pub fn snapshot(&self, member: usize) -> Snapshot {
        let game = self
            .game
            .as_ref()
            .map(|g| g.snapshot(member, member == self.host));
        Snapshot {
            code: self.code.clone(),
            members: (0..self.members.len())
                .map(|m| self.member_view(m))
                .collect(),
            host: self.host,
            you: member,
            revision: self.revision,
            config: self.config.clone(),
            seating: self.seating.clone(),
            game,
            replays: self.replays.clone(),
        }
    }
}
