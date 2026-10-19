use arena_server::{App, api, protocol::Membership};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use futures_util::{SinkExt, StreamExt};
use rand::{Rng, SeedableRng};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::{net::TcpStream, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, connect_async_with_config,
    tungstenite::{Message, client::IntoClientRequest},
};
use tower::ServiceExt;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn crazy_eights_config() -> Value {
    json!({"game":"crazy_eights", "rules": {
        "ace_are_you_strong": true, "jack_skip": true, "ten_reverse": true,
        "two_draw_two": true
    }})
}

struct Server {
    router: Router,
    address: String,
    task: JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Server {
    async fn open(url: &str) -> Self {
        let app = App::open(url, "http://localhost:5173").await.unwrap();
        let router = api::router(app);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let service = router.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, service).await.unwrap();
        });
        Self {
            router,
            address,
            task,
        }
    }
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Value,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        let response = self
            .router
            .clone()
            .oneshot(request.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
    async fn member(&self, kind: &str) -> Membership {
        let (status, value) = self
            .request(
                "POST",
                "/api/rooms",
                json!({"game": kind, "nickname": "Alice"}),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        serde_json::from_value(value).unwrap()
    }
    async fn socket(&self, member: &Membership) -> Socket {
        let (mut socket, _) = connect_async_with_config(
            format!("ws://{}/api/rooms/{}/ws", self.address, member.code),
            None,
            true,
        )
        .await
        .unwrap();
        send(
            &mut socket,
            json!({"type": "authenticate", "token": member.token}),
        )
        .await;
        socket
    }
}

async fn send(socket: &mut Socket, message: Value) {
    socket
        .send(Message::Text(message.to_string().into()))
        .await
        .unwrap();
}
async fn receive(socket: &mut Socket) -> Value {
    timeout(Duration::from_secs(10), async {
        loop {
            match socket.next().await.unwrap().unwrap() {
                Message::Text(text) => return serde_json::from_str(&text).unwrap(),
                Message::Ping(data) => socket.send(Message::Pong(data)).await.unwrap(),
                other => panic!("unexpected {other:?}"),
            }
        }
    })
    .await
    .expect("WebSocket timed out")
}
async fn snapshot(socket: &mut Socket, revision: u64) -> Value {
    loop {
        let message = receive(socket).await;
        assert_eq!(message["type"], "snapshot", "{message}");
        if message["room"]["revision"].as_u64().unwrap() >= revision {
            return message["room"].clone();
        }
    }
}

#[tokio::test]
async fn identity_capacity_and_origin_checks() {
    let server = Server::open("sqlite::memory:").await;
    let alice = server.member("mancala").await;
    let path = format!("/api/rooms/{}", alice.code);
    assert_eq!(
        server.request("GET", &path, Value::Null, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        server
            .request("GET", &path, Value::Null, Some("wrong"))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        server
            .request(
                "POST",
                &format!("{path}/join"),
                json!({"nickname":"alice"}),
                None
            )
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        server
            .request(
                "POST",
                &format!("{path}/join"),
                json!({"nickname":"Bob"}),
                None
            )
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        server
            .request(
                "POST",
                &format!("{path}/join"),
                json!({"nickname":"Eve"}),
                None
            )
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let mut request = format!("ws://{}{path}/ws", server.address)
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("origin", "http://evil.example".parse().unwrap());
    assert!(connect_async(request).await.is_err());
    let (mut socket, _) = connect_async(format!("ws://{}{path}/ws", server.address))
        .await
        .unwrap();
    send(&mut socket, json!({"type":"authenticate", "token":"wrong"})).await;
    assert_eq!(receive(&mut socket).await["type"], "error");
}

#[tokio::test]
async fn queued_setup_actions_from_the_same_revision_both_commit() {
    for kind in ["shithead", "skyjo"] {
        let server = Server::open("sqlite::memory:").await;
        let alice = server.member(kind).await;
        let mut host = server.socket(&alice).await;
        snapshot(&mut host, 1).await;
        let (_, value) = server
            .request(
                "POST",
                &format!("/api/rooms/{}/join", alice.code),
                json!({"nickname":"Bob"}),
                None,
            )
            .await;
        let bob: Membership = serde_json::from_value(value).unwrap();
        let mut guest = server.socket(&bob).await;
        snapshot(&mut host, 2).await;
        snapshot(&mut guest, 2).await;
        send(&mut host, json!({"type":"start", "revision":2})).await;
        let initial = snapshot(&mut host, 3).await;
        let other = snapshot(&mut guest, 3).await;
        assert_eq!(initial["turn"]["players"], json!([0, 1]));
        let host_action = initial["legal_actions"][0].clone();
        let guest_action = other["legal_actions"][0].clone();
        send(
            &mut host,
            json!({"type":"move", "revision":3, "action":host_action}),
        )
        .await;
        send(
            &mut guest,
            json!({"type":"move", "revision":3, "action":guest_action}),
        )
        .await;
        let updated = snapshot(&mut host, 5).await;
        let other = snapshot(&mut guest, 5).await;
        send(
            &mut guest,
            json!({"type":"move", "revision":3, "action":guest_action}),
        )
        .await;
        let error = receive(&mut guest).await;
        assert_eq!(error["type"], "error");
        assert_ne!(error["error"], "Room changed; try again");
        assert_eq!(snapshot(&mut guest, 5).await, other);
        if kind == "shithead" {
            assert_eq!(updated["turn"]["players"], json!([0]));
            assert_eq!(updated["view"]["state"]["setup"], false);
        } else {
            assert_eq!(updated["turn"]["players"], json!([0, 1]));
            assert_eq!(updated["view"]["state"]["phase"], "setup");
        }
    }
}

#[tokio::test]
async fn crew_room_finish_is_host_only_atomic_replayable_and_survives_restart() {
    use sqlx::Connection;
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("crew.db").display());
    let server = Server::open(&database).await;
    let alice = server.member("the_crew").await;
    let path = format!("/api/rooms/{}", alice.code);
    let mut members = vec![alice];
    for name in ["Bob", "Carol"] {
        let (status, value) = server
            .request(
                "POST",
                &format!("{path}/join"),
                json!({"nickname":name}),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        members.push(serde_json::from_value::<Membership>(value).unwrap());
    }
    let mut sockets = vec![];
    for member in &members {
        let mut socket = server.socket(member).await;
        snapshot(&mut socket, 3).await;
        sockets.push(socket);
    }
    send(
        &mut sockets[0],
        json!({"type":"finish","revision":3,"won":true}),
    )
    .await;
    assert_eq!(receive(&mut sockets[0]).await["type"], "error");
    snapshot(&mut sockets[0], 3).await;
    send(&mut sockets[0], json!({"type":"start","revision":3})).await;
    let mut views = vec![];
    for socket in &mut sockets {
        views.push(snapshot(socket, 4).await);
    }
    assert_eq!(views[0]["can_finish"], true);
    assert_eq!(views[1]["can_finish"], false);
    for view in &views {
        assert!(
            view["legal_actions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|a| a["action"]["type"] != "finish")
        );
    }
    send(
        &mut sockets[1],
        json!({"type":"finish","revision":4,"won":true}),
    )
    .await;
    assert_eq!(
        receive(&mut sockets[1]).await["error"],
        "Only the host can finish"
    );
    assert_eq!(snapshot(&mut sockets[1], 4).await, views[1]);
    send(&mut sockets[0],json!({"type":"move","revision":4,"action":{"game":"the_crew","action":{"type":"finish","won":true}}})).await;
    assert_eq!(
        receive(&mut sockets[0]).await["error"],
        "Use the room finish command"
    );
    assert_eq!(snapshot(&mut sockets[0], 4).await, views[0]);
    let player = views[0]["view"]["state"]["next_player"].as_u64().unwrap() as usize;
    let action = views[player]["legal_actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["type"] == "play")
        .unwrap()
        .clone();
    send(
        &mut sockets[player],
        json!({"type":"move","revision":4,"action":action}),
    )
    .await;
    for (index, socket) in sockets.iter_mut().enumerate() {
        views[index] = snapshot(socket, 5).await;
    }
    // A failed finish must publish neither the result nor its replay.
    let mut database_connection = sqlx::SqliteConnection::connect(&database).await.unwrap();
    sqlx::query("CREATE TRIGGER fail_crew_finish BEFORE UPDATE ON rooms BEGIN SELECT RAISE(FAIL, 'forced'); END").execute(&mut database_connection).await.unwrap();
    send(
        &mut sockets[0],
        json!({"type":"finish","revision":4,"won":true}),
    )
    .await;
    assert_eq!(receive(&mut sockets[0]).await["error"], "Storage error");
    assert_eq!(snapshot(&mut sockets[0], 5).await, views[0]);
    sqlx::query("DROP TRIGGER fail_crew_finish")
        .execute(&mut database_connection)
        .await
        .unwrap();
    send(
        &mut sockets[0],
        json!({"type":"finish","revision":4,"won":true}),
    )
    .await;
    for socket in &mut sockets {
        let finished = snapshot(socket, 6).await;
        assert_eq!(finished["outcome"]["winners"], json!([0, 1, 2]));
        assert_eq!(finished["can_finish"], false);
        assert!(finished["legal_actions"].as_array().unwrap().is_empty());
        assert_eq!(finished["replays"].as_array().unwrap().len(), 1);
    }
    drop(sockets);
    drop(server);
    let server = Server::open(&database).await;
    let mut host = server.socket(&members[0]).await;
    let finished = snapshot(&mut host, 6).await;
    let id = finished["replays"][0].as_str().unwrap();
    let (status, value) = server
        .request(
            "GET",
            &format!("{path}/replays/{id}"),
            Value::Null,
            Some(&members[0].token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let replay: arena_core::Replay = serde_json::from_value(value).unwrap();
    assert_eq!(replay.records.len(), 2);
    assert_eq!(
        replay.simulate().unwrap().outcome().unwrap().winners,
        vec![0, 1, 2]
    );
    send(&mut host, json!({"type":"start","revision":6})).await;
    let fresh = snapshot(&mut host, 7).await;
    assert!(fresh["outcome"].is_null());
    assert_eq!(fresh["can_finish"], true);
    send(&mut host, json!({"type":"finish","revision":4,"won":false})).await;
    assert_eq!(receive(&mut host).await["type"], "error");
    snapshot(&mut host, 7).await;
    send(&mut host, json!({"type":"finish","revision":7,"won":false})).await;
    let lost = snapshot(&mut host, 8).await;
    assert_eq!(lost["outcome"]["winners"], json!([]));
    assert_eq!(lost["replays"].as_array().unwrap().len(), 2);
    send(&mut host, json!({"type":"finish","revision":8,"won":false})).await;
    assert_eq!(receive(&mut host).await["type"], "error");
    assert_eq!(snapshot(&mut host, 8).await, lost);
}

#[tokio::test]
async fn games_over_real_websockets_export_and_archive_replays() {
    let server = Server::open("sqlite::memory:").await;
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);
    for kind in [
        "mancala",
        "crazy_eights",
        "shithead",
        "skyjo",
        "tic_tac_toe",
    ] {
        let alice = if kind == "crazy_eights" {
            let (status, value) = server
                .request(
                    "POST",
                    "/api/rooms",
                    json!({
                        "nickname":"Alice", "game":kind, "config":crazy_eights_config()
                    }),
                    None,
                )
                .await;
            assert_eq!(status, StatusCode::CREATED);
            serde_json::from_value(value).unwrap()
        } else {
            server.member(kind).await
        };
        let mut host = server.socket(&alice).await;
        let lobby = snapshot(&mut host, 1).await;
        if kind == "crazy_eights" {
            assert_eq!(lobby["config"], crazy_eights_config());
        }
        let path = format!("/api/rooms/{}", alice.code);
        let (_, value) = server
            .request(
                "POST",
                &format!("{path}/join"),
                json!({"nickname":"Bob"}),
                None,
            )
            .await;
        let bob: Membership = serde_json::from_value(value).unwrap();
        let mut guest = server.socket(&bob).await;
        snapshot(&mut host, 2).await;
        snapshot(&mut guest, 2).await;
        send(&mut guest, json!({"type":"start", "revision":2})).await;
        assert_eq!(receive(&mut guest).await["type"], "error");
        snapshot(&mut guest, 2).await;
        send(&mut host, json!({"type":"start", "revision":1})).await;
        assert_eq!(receive(&mut host).await["type"], "error");
        snapshot(&mut host, 2).await;
        send(&mut host, json!({"type":"start", "revision":2})).await;
        let mut views = [snapshot(&mut host, 3).await, snapshot(&mut guest, 3).await];
        if kind == "crazy_eights" {
            assert_eq!(views[0]["config"], crazy_eights_config());
            assert_eq!(views[1]["config"], crazy_eights_config());
        }
        assert_eq!(
            server
                .request(
                    "POST",
                    &format!("{path}/join"),
                    json!({"nickname":"Late"}),
                    None
                )
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        for _ in 0..20000 {
            if !views[0]["outcome"].is_null() {
                break;
            }
            let player = views[0]["turn"]["players"][0].as_u64().unwrap() as usize;
            let revision = views[0]["revision"].as_u64().unwrap();
            let actions = views[player]["legal_actions"].as_array().unwrap();
            // Make progress rather than voluntarily cycling pickups or replacing revealed cards.
            let candidates: Vec<_> = actions
                .iter()
                .filter(|a| {
                    if kind == "shithead" && actions.len() > 1 {
                        return a["action"]["type"] != "pickup";
                    }
                    if kind == "skyjo" && a["action"]["type"] == "replace" {
                        let index = a["action"]["index"].as_u64().unwrap() as usize;
                        return views[player]["view"]["state"]["grids"][player][index]["value"]
                            .is_null();
                    }
                    true
                })
                .collect();
            let action = if kind == "shithead" {
                let state = &views[player]["view"]["state"];
                let hand = state["hand"].as_array().unwrap();
                let source = if hand.is_empty() {
                    state["players"][player]["up"].as_array().unwrap()
                } else {
                    hand
                };
                candidates
                    .iter()
                    .copied()
                    .filter(|a| a["action"]["type"] == "play")
                    .min_by_key(|a| {
                        let indices = a["action"]["indices"].as_array().unwrap();
                        let rank = source[indices[0].as_u64().unwrap() as usize]["rank"]
                            .as_u64()
                            .unwrap();
                        (
                            match rank {
                                2 => 15,
                                10 => 16,
                                rank => rank,
                            },
                            std::cmp::Reverse(indices.len()),
                        )
                    })
                    .unwrap_or(candidates[rng.gen_range(0..candidates.len())])
            } else {
                candidates[rng.gen_range(0..candidates.len())]
            };
            let socket = if player == 0 { &mut host } else { &mut guest };
            send(
                socket,
                json!({"type":"move", "revision":revision, "action":action}),
            )
            .await;
            views = [
                snapshot(&mut host, revision + 1).await,
                snapshot(&mut guest, revision + 1).await,
            ];
            assert_eq!(views[0]["revision"], views[1]["revision"]);
            assert!(views[0].get("seed").is_none());
        }
        assert!(!views[0]["outcome"].is_null(), "{kind} did not finish");
        let id = views[0]["replays"][0].as_str().unwrap();
        let (status, value) = server
            .request(
                "GET",
                &format!("{path}/replays/{id}"),
                Value::Null,
                Some(&alice.token),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let replay: arena_core::Replay = serde_json::from_value(value).unwrap();
        if kind == "crazy_eights" {
            assert_eq!(replay.version, 4);
            assert_eq!(
                serde_json::to_value(&replay.config).unwrap(),
                crazy_eights_config()
            );
        }
        assert_eq!(
            serde_json::to_value(replay.simulate().unwrap().outcome()).unwrap(),
            views[0]["outcome"]
        );
        // A rematch retains the archived replay.
        let revision = views[0]["revision"].as_u64().unwrap();
        send(&mut host, json!({"type":"start", "revision":revision})).await;
        let fresh = snapshot(&mut host, revision + 1).await;
        assert!(fresh["outcome"].is_null());
        assert_eq!(fresh["config"], lobby["config"]);
        assert_eq!(fresh["replays"].as_array().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn invalid_configuration_is_rejected_before_room_creation() {
    let server = Server::open("sqlite::memory:").await;
    let mut unknown = crazy_eights_config();
    unknown["rules"]["unknown_rule"] = json!(true);
    let mut wrong_type = crazy_eights_config();
    wrong_type["rules"]["two_draw_two"] = json!("true");
    let mut missing = crazy_eights_config();
    missing["rules"]
        .as_object_mut()
        .unwrap()
        .remove("two_draw_two");
    let mut removed_toggle = crazy_eights_config();
    removed_toggle["rules"]["eight_wild"] = json!(false);
    for config in [
        json!({"game":"mancala", "rules":{}}),
        unknown,
        wrong_type,
        missing,
        removed_toggle,
    ] {
        let (status, _) = server
            .request(
                "POST",
                "/api/rooms",
                json!({
                    "nickname":"Alice", "game":"crazy_eights", "config":config
                }),
                None,
            )
            .await;
        assert!(status.is_client_error());
    }
    let alice = server.member("crazy_eights").await;
    let mut socket = server.socket(&alice).await;
    let room = snapshot(&mut socket, 1).await;
    assert!(room["config"]["rules"].get("eight_wild").is_none());
    assert_eq!(room["config"]["rules"]["ace_are_you_strong"], false);
}

#[tokio::test]
async fn configured_lobby_and_game_survive_restart_and_reconnect() {
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("config.db").display());
    let server = Server::open(&database).await;
    let (_, value) = server
        .request(
            "POST",
            "/api/rooms",
            json!({
                "nickname":"Alice", "game":"crazy_eights", "config":crazy_eights_config()
            }),
            None,
        )
        .await;
    let alice: Membership = serde_json::from_value(value).unwrap();
    let path = format!("/api/rooms/{}", alice.code);
    let (_, value) = server
        .request(
            "POST",
            &format!("{path}/join"),
            json!({"nickname":"Bob"}),
            None,
        )
        .await;
    let bob: Membership = serde_json::from_value(value).unwrap();
    drop(server);
    let server = Server::open(&database).await;
    let mut host = server.socket(&alice).await;
    let mut guest = server.socket(&bob).await;
    for room in [snapshot(&mut host, 2).await, snapshot(&mut guest, 2).await] {
        assert_eq!(room["config"], crazy_eights_config());
        assert!(room["view"].is_null());
    }
    // Start cannot alter the agreed options, even for the host.
    send(
        &mut host,
        json!({"type":"start", "revision":2, "config":crazy_eights_config()}),
    )
    .await;
    assert_eq!(receive(&mut host).await["type"], "error");
    assert_eq!(snapshot(&mut host, 2).await["revision"], 2);
    send(&mut host, json!({"type":"start", "revision":2})).await;
    let expected = snapshot(&mut host, 3).await;
    assert_eq!(expected["config"], crazy_eights_config());
    assert_eq!(
        snapshot(&mut guest, 3).await["config"],
        crazy_eights_config()
    );
    host.close(None).await.unwrap();
    guest.close(None).await.unwrap();
    drop(server);
    let server = Server::open(&database).await;
    let mut host = server.socket(&alice).await;
    assert_eq!(snapshot(&mut host, 3).await, expected);
    let (_, room) = server
        .request("GET", &path, Value::Null, Some(&alice.token))
        .await;
    assert_eq!(room, expected);
}

#[tokio::test]
async fn game_and_room_tokens_survive_restart() {
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("restart.db").display());
    let server = Server::open(&database).await;
    let member = server.member("skyjo").await;
    let path = format!("/api/rooms/{}", member.code);
    server
        .request(
            "POST",
            &format!("{path}/join"),
            json!({"nickname":"Bob"}),
            None,
        )
        .await;
    let mut socket = server.socket(&member).await;
    snapshot(&mut socket, 2).await;
    send(&mut socket, json!({"type":"start", "revision":2})).await;
    snapshot(&mut socket, 3).await;
    send(&mut socket, json!({"type":"move", "revision":3, "action":{"game":"skyjo", "action":{"type":"reveal", "index":0}}})).await;
    let expected = snapshot(&mut socket, 4).await;
    socket.close(None).await.unwrap();
    drop(socket);
    drop(server);
    let restarted = Server::open(&database).await;
    let (status, room) = restarted
        .request("GET", &path, Value::Null, Some(&member.token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(room, expected);
    let mut socket = restarted.socket(&member).await;
    assert_eq!(snapshot(&mut socket, 4).await, expected);
}

#[test]
fn openapi_has_distinct_game_schemas() {
    let spec = serde_json::to_value(api::openapi()).unwrap();
    fn check_refs(value: &Value, spec: &Value) {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                    assert!(
                        spec.pointer(reference.strip_prefix('#').unwrap()).is_some(),
                        "missing {reference}"
                    );
                }
                for child in object.values() {
                    check_refs(child, spec);
                }
            }
            Value::Array(array) => {
                for child in array {
                    check_refs(child, spec);
                }
            }
            _ => {}
        }
    }
    check_refs(&spec, &spec);
    for name in [
        "MancalaAction",
        "CrazyEightsAction",
        "ShitheadAction",
        "SkyjoAction",
        "TicTacToeAction",
        "MancalaView",
        "CrazyEightsView",
        "ShitheadView",
        "SkyjoView",
        "TicTacToeView",
        "CrewView",
        "CrewAction",
        "CrewCard",
        "Config",
        "GameConfig",
        "CrazyEightsGameConfig",
    ] {
        assert!(
            spec["components"]["schemas"].get(name).is_some(),
            "missing {name}"
        );
    }
}

#[tokio::test]
async fn failed_persistence_and_now_illegal_commands_do_not_advance_the_room() {
    use sqlx::Connection;
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("atomic.db").display());
    let server = Server::open(&database).await;
    let alice = server.member("mancala").await;
    let path = format!("/api/rooms/{}", alice.code);
    let (_, bob) = server
        .request(
            "POST",
            &format!("{path}/join"),
            json!({"nickname":"Bob"}),
            None,
        )
        .await;
    let bob: Membership = serde_json::from_value(bob).unwrap();
    let mut host = server.socket(&alice).await;
    let mut guest = server.socket(&bob).await;
    snapshot(&mut host, 2).await;
    snapshot(&mut guest, 2).await;
    send(&mut host, json!({"type":"start", "revision":2})).await;
    let initial = snapshot(&mut host, 3).await;
    snapshot(&mut guest, 3).await;
    let command = json!({"type":"move", "revision":3, "action": initial["legal_actions"][0]});

    send(&mut guest, command.clone()).await;
    assert_eq!(receive(&mut guest).await["error"], "Not your turn");
    snapshot(&mut guest, 3).await;

    let mut database = sqlx::SqliteConnection::connect(&database).await.unwrap();
    sqlx::query("CREATE TRIGGER fail_update BEFORE UPDATE ON rooms BEGIN SELECT RAISE(ABORT, 'test write failure'); END;").execute(&mut database).await.unwrap();
    send(&mut host, command.clone()).await;
    assert_eq!(receive(&mut host).await["error"], "Storage error");
    assert_eq!(snapshot(&mut host, 3).await, initial);
    let (_, persisted) = server
        .request("GET", &path, Value::Null, Some(&alice.token))
        .await;
    assert_eq!(persisted, initial);
    let (data,): (String,) = sqlx::query_as("SELECT data FROM rooms WHERE code = ?")
        .bind(&alice.code)
        .fetch_one(&mut database)
        .await
        .unwrap();
    assert_eq!(serde_json::from_str::<Value>(&data).unwrap()["revision"], 3);
    sqlx::query("DROP TRIGGER fail_update")
        .execute(&mut database)
        .await
        .unwrap();

    send(&mut host, command.clone()).await;
    snapshot(&mut host, 4).await;
    snapshot(&mut guest, 4).await;
    send(&mut host, command).await;
    assert_eq!(receive(&mut host).await["error"], "Not your turn");
    let unchanged = snapshot(&mut host, 4).await;
    assert_eq!(unchanged["revision"], 4);
    assert_eq!(unchanged["view"]["state"]["board"][0], 0);
}
