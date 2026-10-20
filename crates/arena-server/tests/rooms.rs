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
    json!({"game":"crazy_eights", "rules": {"ace_are_you_strong":true, "jack_skip":true, "ten_reverse":true, "two_draw_two":true}})
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
    async fn members(&self, count: usize) -> Vec<Membership> {
        let (status, value) = self
            .request("POST", "/api/rooms", json!({"nickname":"Alice"}), None)
            .await;
        assert_eq!(status, StatusCode::CREATED);
        let mut members = vec![serde_json::from_value::<Membership>(value).unwrap()];
        for player in 1..count {
            members.push(
                self.join(&members[0].code, &format!("Member {player}"))
                    .await,
            );
        }
        members
    }
    async fn join(&self, code: &str, name: &str) -> Membership {
        let (status, value) = self
            .request(
                "POST",
                &format!("/api/rooms/{code}/join"),
                json!({"nickname":name}),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK);
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
            json!({"type":"authenticate", "token":member.token}),
        )
        .await;
        socket
    }
    async fn snapshot(&self, member: &Membership) -> Value {
        let (status, value) = self
            .request(
                "GET",
                &format!("/api/rooms/{}", member.code),
                Value::Null,
                Some(&member.token),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        value
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
fn at(room: &Value, mut message: Value) -> Value {
    message["revision"] = room["revision"].clone();
    if message["type"] == "game" || message["type"] == "abort" {
        message["game_id"] = room["game"]["id"].clone();
    }
    message
}
async fn commit(socket: &mut Socket, room: &Value, message: Value) -> Value {
    send(socket, at(room, message)).await;
    loop {
        let response = receive(socket).await;
        assert_eq!(response["type"], "snapshot", "{response}");
        if response["acknowledged"] == true {
            return response["room"].clone();
        }
    }
}
async fn reject(socket: &mut Socket, room: &Value, message: Value) {
    send(socket, at(room, message)).await;
    loop {
        let response = receive(socket).await;
        if response["type"] == "error" {
            break;
        }
        assert_eq!(response["type"], "snapshot");
        assert_eq!(response["acknowledged"], false);
    }
    assert_eq!(
        snapshot(socket, room["revision"].as_u64().unwrap()).await,
        *room
    );
}
async fn start(
    server: &Server,
    members: &[Membership],
    config: Value,
    seats: Vec<usize>,
) -> (Vec<Socket>, Vec<Value>) {
    let mut host = server.socket(&members[0]).await;
    let mut room = snapshot(&mut host, members.len() as u64).await;
    room = commit(
        &mut host,
        &room,
        json!({"type":"configure", "config":config}),
    )
    .await;
    room = commit(&mut host, &room, json!({"type":"seat", "members":seats})).await;
    room = commit(&mut host, &room, json!({"type":"start"})).await;
    let revision = room["revision"].as_u64().unwrap();
    let mut sockets = vec![host];
    let mut views = vec![room];
    for member in &members[1..] {
        let mut socket = server.socket(member).await;
        views.push(snapshot(&mut socket, revision).await);
        sockets.push(socket);
    }
    (sockets, views)
}

#[tokio::test]
async fn game_independent_membership_live_settings_seating_and_permissions() {
    let server = Server::open("sqlite::memory:").await;
    let members = server.members(4).await;
    let mut host = server.socket(&members[0]).await;
    let mut guest = server.socket(&members[1]).await;
    let mut room = snapshot(&mut host, 4).await;
    let mut other = snapshot(&mut guest, 4).await;
    assert!(room["config"].is_null());
    assert!(room["game"].is_null());
    assert_eq!(room["seating"], json!([]));
    assert_eq!(room["members"].as_array().unwrap().len(), 4);
    assert_eq!(room["host"], 0);
    for command in [
        json!({"type":"configure", "config":{"game":"mancala", "rules":{}}}),
        json!({"type":"seat", "members":[1,0]}),
        json!({"type":"start"}),
    ] {
        reject(&mut guest, &other, command).await;
    }
    reject(&mut host, &room, json!({"type":"start"})).await;
    room = commit(
        &mut host,
        &room,
        json!({"type":"configure", "config":crazy_eights_config()}),
    )
    .await;
    other = snapshot(&mut guest, 5).await;
    assert_eq!(other["config"], crazy_eights_config());
    for seats in [json!([0, 0]), json!([4]), json!([usize::MAX])] {
        reject(&mut host, &room, json!({"type":"seat", "members":seats})).await;
    }
    room = commit(&mut host, &room, json!({"type":"seat", "members":[2,1]})).await;
    assert_eq!(room["seating"], json!([2, 1]));
    // Clearing selection does not change membership or seating.
    room = commit(&mut host, &room, json!({"type":"configure", "config":null})).await;
    reject(&mut host, &room, json!({"type":"start"})).await;
    room = commit(
        &mut host,
        &room,
        json!({"type":"configure", "config":{"game":"tic_tac_toe", "rules":{}}}),
    )
    .await;
    for seats in [json!([]), json!([1]), json!([0, 1, 2, 3])] {
        room = commit(&mut host, &room, json!({"type":"seat", "members":seats})).await;
        reject(&mut host, &room, json!({"type":"start"})).await;
    }
    room = commit(&mut host, &room, json!({"type":"seat", "members":[2,1]})).await;
    room = commit(&mut host, &room, json!({"type":"start"})).await;
    let revision = room["revision"].as_u64().unwrap();
    other = snapshot(&mut guest, revision).await;
    assert_eq!(room["game"]["players"][0]["id"], 2);
    assert_eq!(room["game"]["players"][1]["id"], 1);
    assert!(room["game"]["you"].is_null());
    assert!(room["game"]["view"].is_null());
    assert_eq!(room["game"]["legal_actions"], json!([]));
    assert_eq!(other["game"]["you"], 1);
    reject(&mut host, &room, json!({"type":"game", "message":{"type":"move", "action":{"game":"tic_tac_toe", "action":{"index":0}}}})).await;
    reject(&mut host, &room, json!({"type":"seat", "members":[0,1]})).await;
    reject(&mut host, &room, json!({"type":"configure", "config":null})).await;
    reject(&mut guest, &other, json!({"type":"abort"})).await;
    // Joining a full two-player game is independent of game capacity.
    let late = server.join(&members[0].code, "Late").await;
    let late_view = server.snapshot(&late).await;
    assert!(late_view["game"]["view"].is_null());
    assert!(late_view["game"]["you"].is_null());
    assert_eq!(late_view["seating"], json!([2, 1]));
    room = snapshot(&mut host, revision + 1).await;
    room = commit(&mut host, &room, json!({"type":"abort"})).await;
    assert!(room["game"].is_null());
    assert_eq!(room["seating"], json!([2, 1]));
    assert_eq!(room["replays"], json!([]));
    assert_eq!(server.snapshot(&members[0]).await, room);
}

#[tokio::test]
async fn identity_origin_and_malformed_settings_checks() {
    let server = Server::open("sqlite::memory:").await;
    let members = server.members(1).await;
    let path = format!("/api/rooms/{}", members[0].code);
    for token in [None, Some("wrong")] {
        assert_eq!(
            server.request("GET", &path, Value::Null, token).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    for name in ["alice", "", "\n", "1234567890123456789012345"] {
        assert_eq!(
            server
                .request(
                    "POST",
                    &format!("{path}/join"),
                    json!({"nickname":name}),
                    None
                )
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert!(
        server
            .request(
                "POST",
                "/api/rooms",
                json!({"nickname":"X", "game":"mancala"}),
                None
            )
            .await
            .0
            .is_client_error()
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
    let mut socket = server.socket(&members[0]).await;
    let room = snapshot(&mut socket, 1).await;
    let mut unknown = crazy_eights_config();
    unknown["rules"]["unknown"] = json!(true);
    let mut wrong = crazy_eights_config();
    wrong["rules"]["two_draw_two"] = json!("true");
    let mut missing = crazy_eights_config();
    missing["rules"]
        .as_object_mut()
        .unwrap()
        .remove("two_draw_two");
    for config in [
        unknown,
        wrong,
        missing,
        json!({"game":"unknown", "rules":{}}),
    ] {
        reject(
            &mut socket,
            &room,
            json!({"type":"configure", "config":config}),
        )
        .await;
    }
}

#[tokio::test]
async fn queued_same_revision_moves_survive_room_joins_and_reject_old_game_ids() {
    for kind in ["shithead", "skyjo"] {
        let server = Server::open("sqlite::memory:").await;
        let members = server.members(3).await;
        let config = if kind == "shithead" {
            json!({"game":kind,"rules":{"four_card_bombs":false}})
        } else {
            json!({"game":kind,"rules":{}})
        };
        let (mut sockets, mut views) = start(&server, &members, config, vec![2, 1]).await;
        let revision = views[0]["revision"].as_u64().unwrap();
        let id = views[0]["game"]["id"].clone();
        server.join(&members[0].code, "Late").await;
        // Neither the unrelated join nor the earlier game action invalidates these moves.
        for member in [2, 1] {
            send(&mut sockets[member],at(&views[member],json!({"type":"game", "message":{"type":"move", "action":views[member]["game"]["legal_actions"][0]}}))).await;
        }
        for (p, socket) in sockets.iter_mut().enumerate() {
            views[p] = snapshot(socket, revision + 3).await;
        }
        assert_eq!(views[0]["game"]["id"], id);
        assert_eq!(views[0]["revision"], revision + 3);
        assert_ne!(views[2]["game"]["legal_actions"], json!([]));
        let old = at(
            &views[2],
            json!({"type":"game", "message":{"type":"move", "action":views[2]["game"]["legal_actions"][0]}}),
        );
        views[0] = commit(&mut sockets[0], &views[0], json!({"type":"abort"})).await;
        views[0] = commit(&mut sockets[0], &views[0], json!({"type":"start"})).await;
        let fresh = snapshot(&mut sockets[2], revision + 5).await;
        let mut stale = old.clone();
        stale["revision"] = fresh["revision"].clone();
        send(&mut sockets[2], stale).await;
        assert_eq!(receive(&mut sockets[2]).await["type"], "error");
        assert_eq!(snapshot(&mut sockets[2], revision + 5).await, fresh);
        for bad_revision in [revision, revision + 6] {
            let mut message = at(
                &fresh,
                json!({"type":"game", "message":{"type":"move", "action":fresh["game"]["legal_actions"][0]}}),
            );
            message["revision"] = json!(bad_revision);
            send(&mut sockets[2], message).await;
            assert_eq!(receive(&mut sockets[2]).await["type"], "error");
            assert_eq!(snapshot(&mut sockets[2], revision + 5).await, fresh);
        }
    }
}

#[tokio::test]
async fn benched_host_records_crew_results_atomically_and_replays_remain_accessible() {
    use sqlx::Connection;
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("crew.db").display());
    let server = Server::open(&database).await;
    let members = server.members(4).await;
    let (mut sockets, mut views) = start(
        &server,
        &members,
        json!({"game":"the_crew","rules":{}}),
        vec![3, 1, 2],
    )
    .await;
    assert_eq!(views[0]["game"]["can_finish"], true);
    assert!(views[0]["game"]["view"].is_null());
    assert_eq!(views[3]["game"]["you"], 0);
    assert_eq!(views[3]["game"]["can_finish"], false);
    let commander = views[3]["game"]["view"]["state"]["next_player"].clone();
    for member in [3, 1, 2] {
        let state = &views[member]["game"]["view"]["state"];
        assert_eq!(state["commander"], commander);
        assert_eq!(state["communication_played"], json!([false, false, false]));
        assert!(state.get("hands").is_none());
    }
    for view in &views {
        assert!(
            view["game"]["legal_actions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|a| a["action"]["type"] != "finish")
        );
    }
    reject(
        &mut sockets[3],
        &views[3],
        json!({"type":"game","message":{"type":"finish","won":true}}),
    )
    .await;
    reject(&mut sockets[3],&views[3],json!({"type":"game","message":{"type":"move","action":{"game":"the_crew","action":{"type":"finish","won":true}}}})).await;
    let player = views[3]["game"]["view"]["state"]["next_player"]
        .as_u64()
        .unwrap() as usize;
    let member = [3, 1, 2][player];
    let action = views[member]["game"]["legal_actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["action"]["type"] == "play")
        .unwrap()
        .clone();
    let updated = commit(
        &mut sockets[member],
        &views[member],
        json!({"type":"game","message":{"type":"move","action":action}}),
    )
    .await;
    let revision = updated["revision"].as_u64().unwrap();
    for (p, socket) in sockets.iter_mut().enumerate() {
        if p == member {
            views[p] = updated.clone();
        } else {
            views[p] = snapshot(socket, revision).await;
        }
    }
    let mut connection = sqlx::SqliteConnection::connect(&database).await.unwrap();
    sqlx::query(
        "CREATE TRIGGER fail_update BEFORE UPDATE ON rooms BEGIN SELECT RAISE(FAIL, 'forced'); END",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    reject(
        &mut sockets[0],
        &views[0],
        json!({"type":"game","message":{"type":"finish","won":true}}),
    )
    .await;
    sqlx::query("DROP TRIGGER fail_update")
        .execute(&mut connection)
        .await
        .unwrap();
    sqlx::query("CREATE TRIGGER fail_archive BEFORE INSERT ON replays BEGIN SELECT RAISE(FAIL, 'forced'); END")
        .execute(&mut connection).await.unwrap();
    reject(
        &mut sockets[0],
        &views[0],
        json!({"type":"game","message":{"type":"finish","won":true}}),
    )
    .await;
    assert_eq!(server.snapshot(&members[0]).await, views[0]);
    let pending_path = format!(
        "/api/rooms/{}/replays/{}",
        members[0].code,
        views[0]["game"]["id"].as_str().unwrap()
    );
    assert_eq!(
        server
            .request("GET", &pending_path, Value::Null, Some(&members[0].token))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    sqlx::query("DROP TRIGGER fail_archive")
        .execute(&mut connection)
        .await
        .unwrap();
    views[0] = commit(
        &mut sockets[0],
        &views[0],
        json!({"type":"game","message":{"type":"finish","won":true}}),
    )
    .await;
    assert_eq!(views[0]["game"]["outcome"]["winners"], json!([0, 1, 2]));
    assert_eq!(views[0]["replays"].as_array().unwrap().len(), 1);
    assert_eq!(views[0]["replays"][0]["players"][0]["id"], 3);
    assert_eq!(views[0]["game"]["can_finish"], false);
    reject(
        &mut sockets[0],
        &views[0],
        json!({"type":"game","message":{"type":"finish","won":false}}),
    )
    .await;
    let completed = views[0].clone();
    drop(sockets);
    drop(server);
    let server = Server::open(&database).await;
    assert_eq!(server.snapshot(&members[0]).await, completed);
    let restored = server.snapshot(&members[3]).await;
    assert_eq!(restored["game"]["view"]["state"]["commander"], commander);
    assert_eq!(
        restored["game"]["view"]["state"]["communication_played"],
        json!([false, false, false])
    );
    let late = server.join(&members[0].code, "Later arrival").await;
    let path = format!(
        "/api/rooms/{}/replays/{}",
        members[0].code,
        completed["replays"][0]["id"].as_str().unwrap()
    );
    for member in members.iter().chain(std::iter::once(&late)) {
        let (status, value) = server
            .request("GET", &path, Value::Null, Some(&member.token))
            .await;
        assert_eq!(status, StatusCode::OK);
        let replay: arena_core::Replay = serde_json::from_value(value).unwrap();
        assert_eq!(replay.records.len(), 1);
        assert_eq!(
            replay.resolution.as_ref().unwrap().resolution,
            arena_core::Resolution::SharedResult { won: true }
        );
        assert_eq!(replay.resolution.as_ref().unwrap().events.len(), 1);
        assert_eq!(
            replay.simulate().unwrap().outcome().unwrap().winners,
            vec![0, 1, 2]
        );
    }
    assert_eq!(
        server.request("GET", &path, Value::Null, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let mut host = server.socket(&members[0]).await;
    let mut room = snapshot(&mut host, revision + 2).await;
    // A finished game's roster is immutable even while the next game is configured.
    room = commit(&mut host, &room, json!({"type":"seat","members":[0,2]})).await;
    room = commit(
        &mut host,
        &room,
        json!({"type":"configure","config":{"game":"mancala","rules":{}}}),
    )
    .await;
    assert_eq!(room["game"]["players"], completed["game"]["players"]);
    assert_eq!(room["game"]["config"]["game"], "the_crew");
    room = commit(&mut host, &room, json!({"type":"start"})).await;
    assert_eq!(room["game"]["config"]["game"], "mancala");
    assert_eq!(room["game"]["players"][1]["id"], 2);
    assert_ne!(room["game"]["id"], completed["game"]["id"]);
    assert_eq!(room["replays"].as_array().unwrap().len(), 1);
    reject(
        &mut host,
        &room,
        json!({"type":"game","message":{"type":"finish","won":false}}),
    )
    .await;
}

#[tokio::test]
async fn tempel_initial_key_is_configuration_and_restart_preserves_state() {
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("tempel.db").display());
    for count in [3, 10] {
        let server = Server::open(&database).await;
        let members = server.members(count + 1).await;
        let seats = (1..=count).rev().collect();
        let (mut sockets, mut views) = start(
            &server,
            &members,
            json!({"game":"tempel_des_schreckens","rules":{"initial_key":1}}),
            seats,
        )
        .await;
        assert_eq!(views[0]["game"]["config"]["rules"]["initial_key"], 1);
        reject(
            &mut sockets[count],
            &views[count],
            json!({"type":"configure","config":{"game":"tempel_des_schreckens","rules":{"initial_key":2}}}),
        )
        .await;
        reject(&mut sockets[count],&views[count],json!({"type":"game","message":{"type":"move","action":{"game":"tempel_des_schreckens","action":{"type":"choose_key","player":1}}}})).await;
        reject(
            &mut sockets[0],
            &views[0],
            json!({"type":"game","message":{"type":"choose_key","player":1}}),
        )
        .await;
        let revision = views[0]["revision"].as_u64().unwrap();
        for p in 1..=count {
            send(&mut sockets[p],at(&views[p],json!({"type":"game","message":{"type":"move","action":views[p]["game"]["legal_actions"][0]}}))).await;
        }
        let revision = revision + count as u64;
        for (p, socket) in sockets.iter_mut().enumerate() {
            views[p] = snapshot(socket, revision).await;
        }
        assert_eq!(views[1]["game"]["view"]["state"]["key"], 1);
        let saved = views.clone();
        drop(sockets);
        drop(server);
        let server = Server::open(&database).await;
        for (p, member) in members.iter().enumerate() {
            assert_eq!(server.snapshot(member).await, saved[p]);
        }
        let mut socket = server.socket(&members[0]).await;
        let room = snapshot(&mut socket, revision).await;
        let room = commit(&mut socket, &room, json!({"type":"abort"})).await;
        assert!(room["game"].is_null());
        assert!(room["replays"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn games_over_real_websockets_export_and_archive_replays() {
    let server = Server::open("sqlite::memory:").await;
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);
    for (kind, count) in [
        ("mancala", 2),
        ("crazy_eights", 2),
        ("shithead", 2),
        ("skyjo", 2),
        ("tic_tac_toe", 2),
        ("tempel_des_schreckens", 3),
        ("tempel_des_schreckens", 10),
        ("the_crew", 3),
    ] {
        let members = server.members(count).await;
        let config = match kind {
            "crazy_eights" => crazy_eights_config(),
            "shithead" => json!({"game":kind,"rules":{"four_card_bombs":false}}),
            "tempel_des_schreckens" => json!({"game":kind,"rules":{"initial_key":count-1}}),
            _ => json!({"game":kind,"rules":{}}),
        };
        let (mut sockets, mut views) =
            start(&server, &members, config, (0..count).rev().collect()).await;
        for _ in 0..20000 {
            if !views[0]["game"]["outcome"].is_null() {
                break;
            }
            let game = &views[0]["game"];
            let (player, message) = if game["turn"]["players"].as_array().unwrap().is_empty() {
                (0, json!({"type":"finish","won":true}))
            } else {
                let player = game["turn"]["players"][0].as_u64().unwrap() as usize;
                let member = game["players"][player]["id"].as_u64().unwrap() as usize;
                let actions = views[member]["game"]["legal_actions"].as_array().unwrap();
                let candidates: Vec<_> = actions
                    .iter()
                    .filter(|a| {
                        if kind == "shithead" && actions.len() > 1 {
                            return a["action"]["type"] != "pickup";
                        }
                        if kind == "skyjo" && a["action"]["type"] == "replace" {
                            let index = a["action"]["index"].as_u64().unwrap() as usize;
                            return views[member]["game"]["view"]["state"]["grids"][player][index]
                                ["value"]
                                .is_null();
                        }
                        true
                    })
                    .collect();
                let action = if kind == "shithead" {
                    let state = &views[member]["game"]["view"]["state"];
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
                (member, json!({"type":"move","action":action}))
            };
            let revision = views[player]["revision"].as_u64().unwrap();
            let updated = commit(
                &mut sockets[player],
                &views[player],
                json!({"type":"game","message":message}),
            )
            .await;
            for (p, socket) in sockets.iter_mut().enumerate() {
                views[p] = if p == player {
                    updated.clone()
                } else {
                    snapshot(socket, revision + 1).await
                };
            }
        }
        assert!(
            !views[0]["game"]["outcome"].is_null(),
            "{kind} did not finish"
        );
        let id = views[0]["replays"][0]["id"].as_str().unwrap();
        let (status, value) = server
            .request(
                "GET",
                &format!("/api/rooms/{}/replays/{id}", members[0].code),
                Value::Null,
                Some(&members[0].token),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let replay: arena_core::Replay = serde_json::from_value(value).unwrap();
        let replayed = replay.simulate().unwrap();
        for view in &views {
            let p = view["game"]["you"].as_u64().unwrap() as usize;
            assert_eq!(
                serde_json::to_value(replayed.view(p)).unwrap(),
                view["game"]["view"]
            );
            assert_eq!(
                serde_json::to_value(replayed.outcome()).unwrap(),
                view["game"]["outcome"]
            );
            assert_eq!(view["game"]["legal_actions"], json!([]));
        }
        let fresh = commit(&mut sockets[0], &views[0], json!({"type":"start"})).await;
        assert!(fresh["game"]["outcome"].is_null());
        assert_eq!(fresh["game"]["players"], views[0]["game"]["players"]);
        assert_eq!(fresh["replays"].as_array().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn lobby_configuration_and_tokens_survive_restart_and_failed_setup_writes_are_atomic() {
    use sqlx::Connection;
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("setup.db").display());
    let server = Server::open(&database).await;
    let members = server.members(2).await;
    let mut host = server.socket(&members[0]).await;
    let room = snapshot(&mut host, 2).await;
    let mut connection = sqlx::SqliteConnection::connect(&database).await.unwrap();
    sqlx::query(
        "CREATE TRIGGER fail_update BEFORE UPDATE ON rooms BEGIN SELECT RAISE(FAIL, 'forced'); END",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    reject(
        &mut host,
        &room,
        json!({"type":"configure","config":crazy_eights_config()}),
    )
    .await;
    reject(&mut host, &room, json!({"type":"seat","members":[1,0]})).await;
    assert_eq!(server.snapshot(&members[0]).await, room);
    sqlx::query("DROP TRIGGER fail_update")
        .execute(&mut connection)
        .await
        .unwrap();
    let room = commit(
        &mut host,
        &room,
        json!({"type":"configure","config":crazy_eights_config()}),
    )
    .await;
    let room = commit(&mut host, &room, json!({"type":"seat","members":[1,0]})).await;
    sqlx::query(
        "CREATE TRIGGER fail_update BEFORE UPDATE ON rooms BEGIN SELECT RAISE(FAIL, 'forced'); END",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    reject(&mut host, &room, json!({"type":"start"})).await;
    sqlx::query("DROP TRIGGER fail_update")
        .execute(&mut connection)
        .await
        .unwrap();
    drop(host);
    drop(server);
    let server = Server::open(&database).await;
    assert_eq!(server.snapshot(&members[0]).await, room);
    let mut host = server.socket(&members[0]).await;
    assert_eq!(snapshot(&mut host, 4).await, room);
    let mut stale = at(&room, json!({"type":"start"}));
    stale["revision"] = json!(3);
    send(&mut host, stale).await;
    assert_eq!(receive(&mut host).await["type"], "error");
    assert_eq!(snapshot(&mut host, 4).await, room);
    let room = commit(&mut host, &room, json!({"type":"start"})).await;
    drop(host);
    drop(server);
    let server = Server::open(&database).await;
    assert_eq!(server.snapshot(&members[0]).await, room);
    assert_eq!(room["game"]["you"], 1);
    let mut host = server.socket(&members[0]).await;
    let mut guest = server.socket(&members[1]).await;
    snapshot(&mut host, 5).await;
    let other = snapshot(&mut guest, 5).await;
    sqlx::query(
        "CREATE TRIGGER fail_update BEFORE UPDATE ON rooms BEGIN SELECT RAISE(FAIL, 'forced'); END",
    )
    .execute(&mut connection)
    .await
    .unwrap();
    reject(&mut host, &room, json!({"type":"abort"})).await;
    reject(
        &mut guest,
        &other,
        json!({"type":"game","message":{"type":"move","action":other["game"]["legal_actions"][0]}}),
    )
    .await;
    assert_eq!(server.snapshot(&members[0]).await, room);
    sqlx::query("DROP TRIGGER fail_update")
        .execute(&mut connection)
        .await
        .unwrap();
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
        "TempelView",
        "TempelAction",
        "TempelCard",
        "TempelRole",
        "TempelGameConfig",
        "Resolution",
        "ResolutionRecord_Event",
        "Config",
        "GameConfig",
        "CrazyEightsGameConfig",
        "GameMessage",
        "GameSnapshot",
        "MemberView",
        "ReplaySummary",
    ] {
        assert!(
            spec["components"]["schemas"].get(name).is_some(),
            "missing {name}"
        );
    }
}
