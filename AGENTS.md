# Working on Friends' Arena

Read `README.md`, `docs/rules.md`, and `docs/design.md` first. This is a small hobby framework; keep changes focused. No accounts, monetization, decorative UI, or speculative infrastructure.

## Map

- `crates/arena-core/src/game.rs`: `Game` contract, turn/outcome/event types, `Session` validation and accepted-action log.
- `crates/arena-core/src/cards.rs`: generic `Deck<C>` and standard playing cards. Numeric games need not use `Card`; non-card games need not use either.
- `crates/arena-core/src/games/`: independent rulesets and per-player projections.
- `crates/arena-core/src/arena.rs`: runtime game dispatch, transport-friendly action/view enums, versioned replay validation.
- `crates/arena-server/src/room.rs`: anonymous seats, host/start rules, personalized snapshots.
- `crates/arena-server/src/api.rs`: HTTP routes, OpenAPI, authenticated WebSocket lifecycle, serialized room commands.
- `crates/arena-server/src/store.rs`, `migrations/`: SQLite persistence.
- `frontend/src/lib/connection.svelte.ts`: socket authentication, reconnects, revisions, pending state.
- `frontend/src/components/`: game-specific boards. `App.svelte` handles entry, room navigation, and room chrome.
- `frontend/src/lib/api.generated.ts`: generated from `openapi.json`; do not edit by hand.

## Invariants

- Rules stay synchronous and use explicit seeds. Never introduce clocks, OS randomness, network access, nicknames, or database calls into core game transitions.
- `Session::act` must reject illegal, out-of-turn, and post-game actions **before** changing state or history. `Game::advance` is the trusted fast path for actions selected from `legal_actions`.
- Keep turn order inside each ruleset, including extra turns and multi-step phases. A room revision is not a game turn number.
- Only `Game::view` projections and the requesting player's legal actions leave the server. Never serialize a `Match`/`Session` as a live response or expose seeds/stock order/other hands. Skyjo's stock draw is visible only to its active player.
- Replays intentionally contain all hidden information. Only archive a completed game, and only let room members download it. Replay format 1 fixes rules, RNG algorithm, deck order, and action semantics; incompatible changes require a new version and compatibility strategy.
- A room mutex serializes joins and commands. Make a candidate copy, persist it (and any completed replay) in a transaction, then replace memory and notify subscribers. Failed writes must not publish a state change.
- Bearer tokens identify seats, not nicknames or room codes. WebSockets authenticate via the first frame, never a URL query parameter. Never log tokens.
- Every command uses the latest room revision. Reconnects send a fresh snapshot; never automatically resend an unacknowledged move.
- SQL migrations are append-only once released. The current storage model assumes one server process.

## Changing a game

Implement `Game` with its action, event, and view types; reuse `Event` or implement `GameEvent` for a custom event type. Add dispatch in `arena.rs`, board rendering in `GameBoard.svelte`, and a name in `types.ts`. Update `docs/rules.md`. Use unique `#[schema(as = ...)]` names for game-local `Action`/`View` types to avoid OpenAPI collisions. Do not force every game to use all shared utilities.

Add rule-edge tests in the game module, seeded simulations/replay checks in `arena-core/tests`, and server/browser coverage where behavior crosses boundaries. Test malformed indices, rejected-action immutability, hidden information, exhausted decks, termination, and supported player counts.

## Verification

Run Rust format, Clippy with warnings denied, and workspace tests. Run frontend `check`, `build`, and Playwright (commands in README). Protocol changes must regenerate both checked-in files:

```sh
cargo run -q -p arena-server -- --openapi > openapi.json
npm --prefix frontend run api:generate
```

Use `npm --prefix frontend run format` for frontend formatting. Prefer established dependencies to homegrown infrastructure. Avoid adding layers solely for a hypothetical future feature. Keep text and styling restrained, with accessible labels and keyboard controls. Make atomic conventional commits; do not discard unrelated changes.
