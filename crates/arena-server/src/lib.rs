pub mod api;
pub mod error;
mod game;
pub mod protocol;
mod room;
mod store;

use room::Room;
use std::{collections::HashMap, sync::Arc};
use store::Store;
use tokio::sync::{Mutex, broadcast};

pub struct Lobby {
    room: Mutex<Room>,
    changed: broadcast::Sender<()>,
}

impl Lobby {
    fn new(room: Room) -> Self {
        Self {
            room: Mutex::new(room),
            changed: broadcast::channel(16).0,
        }
    }
}

#[derive(Clone)]
pub struct App {
    store: Store,
    rooms: Arc<Mutex<HashMap<String, Arc<Lobby>>>>,
    origin: Arc<str>,
}

impl App {
    pub async fn open(database: &str, origin: &str) -> Result<Self, error::ApiError> {
        let store = Store::open(database).await?;
        let rooms = store
            .load()
            .await?
            .into_iter()
            .map(|room| (room.code.clone(), Arc::new(Lobby::new(room))))
            .collect();
        Ok(Self {
            store,
            rooms: Arc::new(Mutex::new(rooms)),
            origin: origin.into(),
        })
    }
    async fn lobby(&self, code: &str) -> Result<Arc<Lobby>, error::ApiError> {
        self.rooms
            .lock()
            .await
            .get(&code.to_ascii_uppercase())
            .cloned()
            .ok_or(error::ApiError::NotFound)
    }
}
