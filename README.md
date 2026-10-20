# Friends' Arena

A small live-game arena for friends: **Shithead, Crazy Eights, Skyjo, Mancala, Tic-Tac-Toe, The Crew, and Tempel des Schreckens**. Anonymous rooms, nicknames, reconnectable membership, and downloadable replays. No accounts.

## Run locally

Requires current stable Rust, Node **22.12+**, and npm.

From the repository root:

```sh
cargo run -p arena-server
```

In another terminal:

```sh
cd frontend
npm ci
npm run dev
```

Open **http://localhost:5173**. Create a room and share its six-character code. Members join the bench; the host chooses a game, adjusts its live settings, and seats players in order before starting. Settings and seating lock during play. The host can abort or start another game in the same room. Test with separate browser profiles/private windows for different members. Refreshes and temporary disconnections preserve membership; clearing browser storage loses it.

SQLite creates `arena.db` automatically from `crates/arena-server/schema.sql`. Membership, setup, games, and completed replays survive server restarts. Every room member can download completed replays, including benched members. Use a fresh database if stored data does not match the current implementation.

## Deployment

See [DEPLOY.md](DEPLOY.md) for Docker Compose, nginx, HTTPS certificates, configuration, and hosting limitations.

## Game rules

Each game module documents its supported variant, not every house rule:

- [Mancala — Kalah](crates/arena-core/src/games/mancala/rules.md)
- [Crazy Eights](crates/arena-core/src/games/crazy_eights/rules.md)
- [Shithead](crates/arena-core/src/games/shithead/rules.md)
- [Skyjo](crates/arena-core/src/games/skyjo/rules.md)
- [Tic-Tac-Toe](crates/arena-core/src/games/tic_tac_toe/rules.md)
- [The Crew — without missions](crates/arena-core/src/games/the_crew/rules.md) (3–5 players; the host records a shared win or loss)
- [Tempel des Schreckens](crates/arena-core/src/games/tempel_des_schreckens/rules.md) (3–10 players; hidden roles and cards, configurable initial key holder)

Crazy Eights, Mancala, Shithead, and Skyjo use replay version 4; Tic-Tac-Toe, The Crew, and Tempel des Schreckens use version 1. Back up databases and use separate databases (`DATABASE_URL`) for incompatible rules versions. See [compatibility requirements and revision references](docs/design.md#replay-and-save-compatibility).

## Develop

- `crates/arena-core`: deterministic rules, shared cards/decks, `Game` trait, validated sessions, action/event logs, player views, replays. No I/O or async runtime.
- `crates/arena-server`: Axum/Tokio, SQLite, anonymous membership, personalized per-room WebSockets.
- `frontend`: Svelte 5 SPA, generated API types, one board component per game.
- [Frontend components and languages](frontend/README.md) · [Frontend design](docs/frontend-design.md)
- [Add a game](docs/adding-a-game.md) · [Design and `../spiele` review](docs/design.md) · [Wire protocol](docs/protocol.md) · [Agent guidance](AGENTS.md)

OpenAPI: [`openapi.json`](openapi.json), also served at `/api/openapi.json`. After changing API types/routes:

```sh
cargo run -q -p arena-server -- --openapi > openapi.json
npm --prefix frontend run api:generate
```

Checks:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
npm --prefix frontend run check
npm --prefix frontend run build
cd frontend
npx playwright install chromium
npm test
```

Playwright starts isolated in-memory backend and Vite processes on ports 13000 and 15173, leaving development servers alone. Those test ports must be free. Backend integration tests use real WebSockets and temporary SQLite databases.

Run headless games or validate an exported replay:

```sh
cargo run --release -p arena-core --example simulate -- skyjo 1000
cargo run -p arena-core --example replay -- path/to/replay.json
```
