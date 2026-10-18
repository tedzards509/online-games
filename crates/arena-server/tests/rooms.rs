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
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tower::ServiceExt;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

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
        let (mut socket, _) = connect_async(format!(
            "ws://{}/api/rooms/{}/ws",
            self.address, member.code
        ))
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
async fn four_games_over_real_websockets_persist_and_export_replays() {
    let directory = tempfile::tempdir().unwrap();
    let database = format!("sqlite://{}", directory.path().join("test.db").display());
    let server = Server::open(&database).await;
    let mut memberships = vec![];
    let mut rng = rand_chacha_for_test();
    for kind in ["mancala", "crazy_eights", "shithead", "skyjo"] {
        let alice = server.member(kind).await;
        let mut host = server.socket(&alice).await;
        snapshot(&mut host, 1).await;
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
            let player = views[0]["turn"]["player"].as_u64().unwrap() as usize;
            let revision = views[0]["revision"].as_u64().unwrap();
            let actions = views[player]["legal_actions"].as_array().unwrap();
            let action = &actions[rng.gen_range(0..actions.len())];
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
        assert_eq!(
            serde_json::to_value(replay.simulate().unwrap().outcome()).unwrap(),
            views[0]["outcome"]
        );
        // A rematch retains the archived replay.
        let revision = views[0]["revision"].as_u64().unwrap();
        send(&mut host, json!({"type":"start", "revision":revision})).await;
        let fresh = snapshot(&mut host, revision + 1).await;
        assert!(fresh["outcome"].is_null());
        assert_eq!(fresh["replays"].as_array().unwrap().len(), 1);
        memberships.push(alice);
    }
    drop(server);
    let restarted = Server::open(&database).await;
    for member in memberships {
        let (status, room) = restarted
            .request(
                "GET",
                &format!("/api/rooms/{}", member.code),
                Value::Null,
                Some(&member.token),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert!(!room["view"].is_null());
        let mut socket = restarted.socket(&member).await;
        let live = snapshot(&mut socket, room["revision"].as_u64().unwrap()).await;
        assert_eq!(live, room);
    }
}

fn rand_chacha_for_test() -> rand::rngs::StdRng {
    rand::rngs::StdRng::seed_from_u64(7)
}

#[test]
fn openapi_has_distinct_game_schemas() {
    let spec = serde_json::to_value(api::openapi()).unwrap();
    for name in [
        "MancalaAction",
        "CrazyEightsAction",
        "ShitheadAction",
        "SkyjoAction",
        "MancalaView",
        "CrazyEightsView",
        "ShitheadView",
        "SkyjoView",
    ] {
        assert!(
            spec["components"]["schemas"].get(name).is_some(),
            "missing {name}"
        );
    }
}
