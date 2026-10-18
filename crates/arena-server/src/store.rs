use crate::{error::ApiError, room::Room};
use arena_core::Replay;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn open(url: &str) -> Result<Self, ApiError> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
    pub async fn load(&self) -> Result<Vec<Room>, ApiError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT data FROM rooms")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|(data,)| Ok(serde_json::from_str(&data)?))
            .collect()
    }
    pub async fn save(&self, room: &Room, replay: Option<(&str, &Replay)>) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO rooms (code, data) VALUES (?, ?) ON CONFLICT(code) DO UPDATE SET data = excluded.data")
            .bind(&room.code).bind(serde_json::to_string(room)?).execute(&mut *tx).await?;
        if let Some((id, replay)) = replay {
            sqlx::query("INSERT INTO replays (id, room_code, data) VALUES (?, ?, ?)")
                .bind(id)
                .bind(&room.code)
                .bind(serde_json::to_string(replay)?)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn replay(&self, code: &str, id: &str) -> Result<Replay, ApiError> {
        let (data,): (String,) =
            sqlx::query_as("SELECT data FROM replays WHERE id = ? AND room_code = ?")
                .bind(id)
                .bind(code)
                .fetch_optional(&self.pool)
                .await?
                .ok_or(ApiError::NotFound)?;
        Ok(serde_json::from_str(&data)?)
    }
}
