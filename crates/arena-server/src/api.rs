use crate::{
    App, Lobby,
    error::{ApiError, ErrorBody},
    protocol::*,
    room::Room,
};
use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use rand::Rng;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::broadcast,
    time::{interval, timeout},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        OpenApi as Spec,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Friends' Arena", version = "0.1.0", description = "Anonymous rooms. HTTP uses bearer room tokens. WebSockets authenticate in the first frame; see ClientMessage and ServerMessage schemas."),
    paths(create_room, join_room, room_snapshot, replay, websocket, health),
    components(schemas(CreateRoom, JoinRoom, Membership, Snapshot, ClientMessage, ServerMessage, ErrorBody, arena_core::Replay)),
    modifiers(&Security)
)]
struct Api;
struct Security;
impl Modify for Security {
    fn modify(&self, spec: &mut Spec) {
        spec.components.as_mut().unwrap().add_security_scheme(
            "room_token",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );
    }
}
pub fn openapi() -> Spec {
    let mut spec = Api::openapi();
    spec.info.license = None;
    spec
}

pub fn router(app: App) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/openapi.json", get(|| async { Json(openapi()) }))
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/{code}/join", post(join_room))
        .route("/api/rooms/{code}", get(room_snapshot))
        .route("/api/rooms/{code}/ws", get(websocket))
        .route("/api/rooms/{code}/replays/{id}", get(replay))
        .layer(DefaultBodyLimit::max(4096))
        .layer(TraceLayer::new_for_http())
        .fallback_service(
            ServeDir::new("frontend/dist")
                .not_found_service(ServeFile::new("frontend/dist/index.html")),
        )
        .with_state(app)
}

#[utoipa::path(get, path = "/api/health", responses((status = 204, description = "Ready")))]
async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[utoipa::path(post, path = "/api/rooms", request_body = CreateRoom, responses((status = 201, body = Membership), (status = 400, body = ErrorBody)))]
async fn create_room(
    State(app): State<App>,
    Json(request): Json<CreateRoom>,
) -> Result<impl IntoResponse, ApiError> {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rooms = app.rooms.lock().await;
    let code = loop {
        let candidate: String = (0..6)
            .map(|_| ALPHABET[rand::thread_rng().gen_range(0..ALPHABET.len())] as char)
            .collect();
        if !rooms.contains_key(&candidate) {
            break candidate;
        }
    };
    let mut room = Room::new(code.clone());
    let membership = room.join(&request.nickname)?;
    app.store.save(&room, None).await?;
    rooms.insert(code, Arc::new(Lobby::new(room)));
    Ok((StatusCode::CREATED, Json(membership)))
}

#[utoipa::path(post, path = "/api/rooms/{code}/join", params(("code" = String, Path)), request_body = JoinRoom, responses((status = 200, body = Membership), (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
async fn join_room(
    State(app): State<App>,
    Path(code): Path<String>,
    Json(request): Json<JoinRoom>,
) -> Result<Json<Membership>, ApiError> {
    let lobby = app.lobby(&code).await?;
    let mut room = lobby.room.lock().await;
    let mut next = room.clone();
    let membership = next.join(&request.nickname)?;
    app.store.save(&next, None).await?;
    *room = next;
    let _ = lobby.changed.send(());
    Ok(Json(membership))
}

fn bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)
}

#[utoipa::path(get, path = "/api/rooms/{code}", params(("code" = String, Path)), security(("room_token" = [])), responses((status = 200, body = Snapshot), (status = 401, body = ErrorBody), (status = 404, body = ErrorBody)))]
async fn room_snapshot(
    State(app): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Snapshot>, ApiError> {
    let lobby = app.lobby(&code).await?;
    let room = lobby.room.lock().await;
    let member = room.authenticate(bearer(&headers)?)?;
    Ok(Json(room.snapshot(member)))
}

#[utoipa::path(get, path = "/api/rooms/{code}/replays/{id}", params(("code" = String, Path), ("id" = String, Path)), security(("room_token" = [])), responses((status = 200, body = arena_core::Replay), (status = 401, body = ErrorBody), (status = 404, body = ErrorBody)))]
async fn replay(
    State(app): State<App>,
    Path((code, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<arena_core::Replay>, ApiError> {
    let lobby = app.lobby(&code).await?;
    let code = {
        let room = lobby.room.lock().await;
        room.authenticate(bearer(&headers)?)?;
        room.code.clone()
    };
    Ok(Json(app.store.replay(&code, &id).await?))
}

#[utoipa::path(get, path = "/api/rooms/{code}/ws", params(("code" = String, Path)), responses((status = 101, description = "Per-room WebSocket. Authenticate first, then send room commands or nested game messages. Receives personalized snapshots and errors."), (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
async fn websocket(
    State(app): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    if headers
        .get("origin")
        .is_some_and(|origin| origin.as_bytes() != app.origin.as_bytes())
    {
        return Err(ApiError::BadRequest("Origin not allowed".into()));
    }
    let lobby = app.lobby(&code).await?;
    Ok(ws
        .max_message_size(4096)
        .max_frame_size(4096)
        .on_upgrade(move |socket| connection(socket, app, lobby)))
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(message).map_err(|_| ())?;
    timeout(
        Duration::from_secs(10),
        socket.send(Message::Text(text.into())),
    )
    .await
    .map_err(|_| ())?
    .map_err(|_| ())
}

async fn connection(mut socket: WebSocket, app: App, lobby: Arc<Lobby>) {
    let member = match timeout(Duration::from_secs(5), socket.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Authenticate { token }) => {
                lobby.room.lock().await.authenticate(&token)
            }
            _ => Err(ApiError::Unauthorized),
        },
        _ => Err(ApiError::Unauthorized),
    };
    let member = match member {
        Ok(member) => member,
        Err(error) => {
            let _ = send(
                &mut socket,
                &ServerMessage::Error {
                    error: error.to_string(),
                },
            )
            .await;
            return;
        }
    };
    let mut changes = lobby.changed.subscribe();
    let snapshot = lobby.room.lock().await.snapshot(member);
    let mut last_revision = snapshot.revision;
    if send(
        &mut socket,
        &ServerMessage::Snapshot {
            room: Box::new(snapshot),
            acknowledged: false,
        },
    )
    .await
    .is_err()
    {
        return;
    }
    let mut heartbeat = interval(Duration::from_secs(20));
    heartbeat.tick().await;
    let mut alive = true;
    loop {
        tokio::select! {
            incoming = socket.next() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    let result = match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(message) => command(&app, &lobby, member, message).await,
                        Err(_) => Err(ApiError::BadRequest("Invalid message".into())),
                    };
                    let acknowledged = result.is_ok();
                    if let Err(error) = result
                        && send(&mut socket, &ServerMessage::Error { error: error.to_string() }).await.is_err() { break; }
                    let room = lobby.room.lock().await.snapshot(member);
                    last_revision = room.revision;
                    if send(&mut socket, &ServerMessage::Snapshot { room: Box::new(room), acknowledged }).await.is_err() { break; }
                }
                Some(Ok(Message::Ping(data))) => { if socket.send(Message::Pong(data)).await.is_err() { break; } }
                Some(Ok(Message::Pong(_))) => alive = true,
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                _ => break,
            },
            change = changes.recv() => {
                if matches!(change, Err(broadcast::error::RecvError::Closed)) { break; }
                let room = lobby.room.lock().await.snapshot(member);
                if room.revision > last_revision {
                    last_revision = room.revision;
                    if send(&mut socket, &ServerMessage::Snapshot { room: Box::new(room), acknowledged: false }).await.is_err() { break; }
                }
            }
            _ = heartbeat.tick() => {
                if !alive { break; } alive = false;
                if socket.send(Message::Ping(vec![].into())).await.is_err() { break; }
            }
        }
    }
}

async fn command(
    app: &App,
    lobby: &Lobby,
    member: usize,
    message: ClientMessage,
) -> Result<(), ApiError> {
    // Tokio's mutex queues lock acquisitions FIFO. Each move is validated only
    // when it reaches the head, against the last successfully persisted state.
    let mut room = lobby.room.lock().await;
    let revision = match &message {
        ClientMessage::Configure { revision, .. }
        | ClientMessage::Seat { revision, .. }
        | ClientMessage::Start { revision }
        | ClientMessage::Abort { revision, .. }
        | ClientMessage::Game { revision, .. } => *revision,
        _ => return Err(ApiError::BadRequest("Already authenticated".into())),
    };
    let invalid_revision = match &message {
        ClientMessage::Configure { .. }
        | ClientMessage::Seat { .. }
        | ClientMessage::Start { .. }
        | ClientMessage::Abort { .. } => revision != room.revision,
        ClientMessage::Game { game_id, .. } => {
            let game = room.check_game(game_id)?;
            revision > room.revision || revision < game.started_at
        }
        _ => unreachable!(),
    };
    if invalid_revision {
        return Err(ApiError::Conflict);
    }
    let mut next = room.clone();
    let mut replay = None;
    match message {
        ClientMessage::Configure { config, .. } => next.configure(member, config)?,
        ClientMessage::Seat { members, .. } => next.seat(member, members)?,
        ClientMessage::Start { .. } => next.start(member)?,
        ClientMessage::Abort { game_id, .. } => next.abort(member, &game_id)?,
        ClientMessage::Game { message, .. } => {
            next.act(member, message)?;
            if let Some(game) = next.game.as_ref().filter(|g| g.session.outcome().is_some()) {
                replay = Some((game.id.clone(), game.session.replay()));
                next.replays.push(game.replay_summary());
            }
        }
        ClientMessage::Authenticate { .. } => unreachable!(),
    }
    app.store
        .save(
            &next,
            replay.as_ref().map(|(id, replay)| (id.as_str(), replay)),
        )
        .await?;
    *room = next;
    let _ = lobby.changed.send(());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arena_core::{
        Action, GameKind,
        games::{skyjo, tic_tac_toe},
    };

    fn configured_room(kind: GameKind, count: usize) -> Room {
        let mut room = Room::new("ABC234".into());
        for p in 0..count {
            room.join(&format!("Member {p}")).unwrap();
        }
        room.configure(0, Some(kind.default_config())).unwrap();
        room.seat(0, (0..count).collect()).unwrap();
        room
    }

    fn movement(id: &str, revision: u64, action: Action) -> ClientMessage {
        ClientMessage::Game {
            game_id: id.into(),
            revision,
            message: GameMessage::Move { action },
        }
    }

    #[tokio::test]
    async fn crew_communication_and_lead_are_revalidated_in_queue_order() {
        for communication_first in [false, true] {
            let app = App::open("sqlite::memory:", "http://localhost:5173")
                .await
                .unwrap();
            let mut room = configured_room(GameKind::TheCrew, 3);
            room.start(0).unwrap();
            let id = room.game.as_ref().unwrap().id.clone();
            let arena_core::View::TheCrew(view) = room.game.as_ref().unwrap().session.view(0)
            else {
                unreachable!()
            };
            let leader = view.next_player;
            let speaker = (leader + 1) % 3;
            let revision = room.revision;
            let play = room
                .snapshot(leader)
                .game
                .unwrap()
                .legal_actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::TheCrew(arena_core::games::the_crew::Action::Play { .. })
                    )
                })
                .unwrap();
            let communication = room
                .snapshot(speaker)
                .game
                .unwrap()
                .legal_actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::TheCrew(arena_core::games::the_crew::Action::Communicate { .. })
                    )
                })
                .unwrap();
            app.store.save(&room, None).await.unwrap();
            let lobby = Lobby::new(room);
            let lead = movement(&id, revision, play);
            let communicate = movement(&id, revision, communication);
            let (first, second) = if communication_first {
                tokio::join!(biased; command(&app,&lobby,speaker,communicate),command(&app,&lobby,leader,lead))
            } else {
                tokio::join!(biased; command(&app,&lobby,leader,lead),command(&app,&lobby,speaker,communicate))
            };
            first.unwrap();
            assert_eq!(second.is_ok(), communication_first);
            let room = lobby.room.lock().await;
            assert_eq!(
                room.revision,
                revision + if communication_first { 2 } else { 1 }
            );
            let arena_core::View::TheCrew(view) = room.game.as_ref().unwrap().session.view(0)
            else {
                unreachable!()
            };
            assert_eq!(view.communications[speaker].is_some(), communication_first);
            assert_eq!(view.trick.len(), 1);
            assert_eq!(
                room.snapshot(0).game.unwrap().turn.players,
                vec![view.next_player]
            );
            assert!(room.snapshot(0).game.unwrap().can_finish);
            assert_eq!(
                room.game
                    .as_ref()
                    .unwrap()
                    .session
                    .replay()
                    .simulate()
                    .unwrap()
                    .replay(),
                room.game.as_ref().unwrap().session.replay()
            );
        }
    }

    #[tokio::test]
    async fn queued_eligible_players_both_commit_from_the_same_revision() {
        for kind in [GameKind::Shithead, GameKind::Skyjo] {
            let app = App::open("sqlite::memory:", "http://localhost:5173")
                .await
                .unwrap();
            let mut room = configured_room(kind, 2);
            room.start(0).unwrap();
            let id = room.game.as_ref().unwrap().id.clone();
            let alice_action = room.snapshot(0).game.unwrap().legal_actions[0].clone();
            let bob_action = room.snapshot(1).game.unwrap().legal_actions[0].clone();
            app.store.save(&room, None).await.unwrap();
            let revision = room.revision;
            let lobby = Lobby::new(room);
            let mut changes = lobby.changed.subscribe();
            let (alice, bob) = tokio::join!(
                biased;
                command(
                    &app,
                    &lobby,
                    0,
                    movement(&id, revision, alice_action)
                ),
                command(
                    &app,
                    &lobby,
                    1,
                    movement(&id, revision, bob_action)
                ),
            );
            alice.unwrap();
            bob.unwrap();
            let room = lobby.room.lock().await;
            assert_eq!(room.revision, revision + 2);
            let replay = room.game.as_ref().unwrap().session.replay();
            assert_eq!(
                replay.records.iter().map(|r| r.player).collect::<Vec<_>>(),
                vec![0, 1]
            );
            assert_eq!(
                app.store.load().await.unwrap()[0]
                    .game
                    .as_ref()
                    .unwrap()
                    .session
                    .replay(),
                replay
            );
            changes.try_recv().unwrap();
            changes.try_recv().unwrap();
            assert!(matches!(
                changes.try_recv(),
                Err(broadcast::error::TryRecvError::Empty)
            ));
        }
    }

    #[tokio::test]
    async fn an_illegal_queued_action_does_not_block_the_next_legal_action() {
        let app = App::open("sqlite::memory:", "http://localhost:5173")
            .await
            .unwrap();
        let mut room = configured_room(GameKind::Skyjo, 2);
        room.start(0).unwrap();
        let id = room.game.as_ref().unwrap().id.clone();
        app.store.save(&room, None).await.unwrap();
        let revision = room.revision;
        let lobby = Lobby::new(room);
        let reveal = || {
            movement(
                &id,
                revision,
                Action::Skyjo(skyjo::Action::Reveal { index: 0 }),
            )
        };
        let (first, invalid, next) = tokio::join!(biased;
            command(&app, &lobby, 0, reveal()),
            command(&app, &lobby, 0, reveal()),
            command(&app, &lobby, 1, reveal()),
        );
        first.unwrap();
        assert!(matches!(invalid, Err(ApiError::BadRequest(_))));
        next.unwrap();
        let room = lobby.room.lock().await;
        assert_eq!(room.revision, revision + 2);
        assert_eq!(room.game.as_ref().unwrap().session.action_count(), 2);
        assert_eq!(
            app.store.load().await.unwrap()[0]
                .game
                .as_ref()
                .unwrap()
                .session
                .replay(),
            room.game.as_ref().unwrap().session.replay()
        );
    }

    #[tokio::test]
    async fn future_revisions_and_actions_from_previous_games_are_rejected() {
        let app = App::open("sqlite::memory:", "http://localhost:5173")
            .await
            .unwrap();
        let mut room = configured_room(GameKind::TicTacToe, 2);
        room.start(0).unwrap();
        let id = room.game.as_ref().unwrap().id.clone();
        app.store.save(&room, None).await.unwrap();
        let revision = room.revision;
        let lobby = Lobby::new(room);
        for (player, index) in [(0, 0), (1, 3), (0, 1), (1, 4), (0, 2)] {
            command(
                &app,
                &lobby,
                player,
                movement(
                    &id,
                    revision,
                    Action::TicTacToe(tic_tac_toe::Action { index }),
                ),
            )
            .await
            .unwrap();
        }
        assert!(matches!(
            command(&app, &lobby, 0, ClientMessage::Start { revision }).await,
            Err(ApiError::Conflict)
        ));
        let current = lobby.room.lock().await.revision;
        command(&app, &lobby, 0, ClientMessage::Start { revision: current })
            .await
            .unwrap();
        let before = serde_json::to_value(&*lobby.room.lock().await).unwrap();
        for revision in [revision, current + 2] {
            assert!(matches!(
                command(
                    &app,
                    &lobby,
                    0,
                    movement(
                        &id,
                        revision,
                        Action::TicTacToe(tic_tac_toe::Action { index: 0 })
                    )
                )
                .await,
                Err(ApiError::Conflict)
            ));
            assert_eq!(
                serde_json::to_value(&*lobby.room.lock().await).unwrap(),
                before
            );
        }
    }
}
