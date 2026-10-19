# Friends' Arena

A small live-game arena for friends: **Shithead, Crazy Eights, Skyjo, Mancala, Tic-Tac-Toe, and The Crew**. Anonymous rooms, nicknames, reconnectable seats, and downloadable replays. No accounts.

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

Open **http://localhost:5173**. Choose a game and its rule options before creating a room, share its six-character code, and start once everyone has joined. Everyone sees the fixed room rules; rematches keep them. Test with separate browser profiles/private windows to occupy different seats. A refresh or temporary disconnection preserves your seat; clearing browser storage loses it.

SQLite creates `arena.db` automatically. Games, room tokens (hashed), and completed replays survive server restarts. The host can start a rematch; previous replays remain available.

## Hosting for friends

```sh
npm --prefix frontend run build
PUBLIC_ORIGIN=https://cards.example.org BIND_ADDR=127.0.0.1:3000 cargo run --release -p arena-server
```

Run from the repository root. The server serves `frontend/dist` with SPA routing. Put it behind an HTTPS reverse proxy that forwards WebSocket upgrades. Alternatively, host the SPA separately and proxy its `/api` routes to Rust. `PUBLIC_ORIGIN` must exactly match the browser's origin (scheme, hostname, port; no trailing slash). For LAN development, set it to `http://<your-ip>:5173`.

Configuration: `DATABASE_URL` (default `sqlite://arena.db`), `BIND_ADDR` (default `127.0.0.1:3000`), `PUBLIC_ORIGIN` (default `http://localhost:5173`), `RUST_LOG`.

This is a **single-server, trusted-friends foundation**, not a public service: no room expiry, seat removal/host transfer, spectators, or abuse rate limiting. Do not run multiple server processes against the same database. Room codes are invitations, not secrets; seat tokens are secrets. Back up the database; it contains hidden game state. Skyjo currently plays one round, not a cumulative match to 100.

## Game rules

Each game module documents its supported variant, not every house rule:

- [Mancala — Kalah](crates/arena-core/src/games/mancala/rules.md)
- [Crazy Eights](crates/arena-core/src/games/crazy_eights/rules.md)
- [Shithead](crates/arena-core/src/games/shithead/rules.md)
- [Skyjo](crates/arena-core/src/games/skyjo/rules.md)
- [Tic-Tac-Toe](crates/arena-core/src/games/tic_tac_toe/rules.md)
- [The Crew — without missions](crates/arena-core/src/games/the_crew/rules.md) (3–5 players; the host records a shared win or loss)

Crazy Eights, Mancala, Shithead, and Skyjo use replay version 4; Tic-Tac-Toe and The Crew use version 1. Back up databases and use separate databases (`DATABASE_URL`) for incompatible rules versions. See [compatibility requirements and revision references](docs/design.md#replay-and-save-compatibility).

## Develop

- `crates/arena-core`: deterministic rules, shared cards/decks, `Game` trait, validated sessions, action/event logs, player views, replays. No I/O or async runtime.
- `crates/arena-server`: Axum/Tokio, SQLite, anonymous membership, personalized per-room WebSockets.
- `frontend`: Svelte 5 SPA, generated API types, one board component per game.
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
