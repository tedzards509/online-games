# Working on Friends' Arena

Read `README.md`, `docs/design.md`, and the relevant game's `crates/arena-core/src/games/<id>/rules.md` first. This is a small hobby framework to refine collaboratively with the user, not a finished product to embellish autonomously. Keep changes focused on the requested feature. No accounts, monetization, unsolicited decorative UI, or speculative infrastructure.

## Map

- `crates/arena-core/src/game.rs`: `Game` contract, turn/outcome/event types, `Session` validation and accepted-action log.
- `crates/arena-core/src/cards.rs`: generic `Deck<C>` and standard playing cards. Numeric games need not use `Card`; non-card games need not use either.
- `crates/arena-core/src/games/<id>/`: independent rulesets and per-player projections in `mod.rs`, with game rules in `rules.md`.
- `crates/arena-core/src/arena.rs`: runtime game dispatch, transport-friendly action/view enums, versioned replay validation.
- `crates/arena-server/src/room.rs`: stable Members, bench/seating, live setup, game lifecycle, room snapshots.
- `crates/arena-server/src/game.rs`: frozen game roster, member-to-player routing, host-only game controls, player snapshots.
- `crates/arena-server/src/api.rs`: HTTP routes, OpenAPI, authenticated WebSocket lifecycle, serialized room commands.
- `crates/arena-server/src/store.rs`, `schema.sql`: SQLite persistence and direct schema initialization.
- `frontend/src/lib/connection.svelte.ts`: socket authentication, reconnects, revisions, pending state.
- `frontend/src/components/`: game-specific boards. `App.svelte` handles entry, room navigation, and room chrome.
- `frontend/src/lib/api.generated.ts`: generated from `openapi.json`; do not edit by hand.

## Code style

- Value understandable, idiomatic code: model domain concepts explicitly and prefer clear names over magic numbers or clever shortcuts.
- Keep code modular, minimal, and clean. Avoid boilerplate, redundant wrappers, speculative abstraction layers, and filler code or prose ("slop"). Prefer direct, readable implementations.
- Before implementing general-purpose functionality, look for a well-maintained, popular library that meets the requirements. Use it rather than writing a custom equivalent when it covers the need.
- Separate reusable game contracts from independent rulesets, server orchestration, and board rendering. Reuse cards, decks, and events where useful; do not constrain every game to the same representation.
- Model actions as commands, events as resulting effects, state as authoritative data, and views as player-specific projections. Keep these distinctions explicit; document the model in `docs/design.md`.
- Do not assume turns are inherently single-player. Use `Turn.players` for eligibility and pass the acting seat explicitly to rules. Rulesets choose the next eligible seats; the server queues moves FIFO under the room mutex and validates each against the latest committed state. An older observed revision alone does not invalidate a still-legal move in the same game. `Turn::advance` is a singleton-only helper and must panic otherwise.
- Document shared core functions, types, and constants with short Rustdoc descriptions of what they do. Add a short module docstring to every non-game `arena-core` module; the `games*` modules are exempt from that blanket requirement.
- Keep technical documentation concise and precise: definitions, contracts, invariants, and actionable steps rather than narrative. Update the new-game guide when extension steps change, and validate shared extension points with a small working game.
- Comments and documentation must describe the status quo: behavior, contracts, and constraints, not implementation history or comparisons with past behavior. Never use change-relative phrasing. Keep change summaries in Git commits; describe compatibility using explicit versions and requirements.
- Minimize frontend text: include only what is needed to play and operate the app, while preserving accessible labels and keyboard controls. Keep styling functional; add beautification only when the user explicitly requests it, feature by feature.
- Keep `AGENTS.md` useful for navigating and modifying the codebase, and `README.md` concise, human-readable, and accurate about how to run it. Update them when relevant workflows change.
- Work with Git. Make understandable, atomic commits with conventional commit messages; keep unrelated changes separate and never discard the user's work.
- Commit by purpose, not by layer: one fix, feature, or standalone documentation update/refactor per commit. Keep the backend, frontend, generated files, and tests needed for the same change together; keep unrelated changes separate.

## Invariants

- Rules stay synchronous and use explicit seeds. Never introduce clocks, OS randomness, network access, nicknames, or database calls into core game transitions.
- `Session::act` must reject illegal, out-of-turn, and post-game actions **before** changing state or history. `Game::advance` is the trusted fast path for actions selected from `legal_actions`.
- Keep turn order inside each ruleset, including extra turns and multi-step phases. A room revision is not a game turn number.
- Only public setup configuration, `Game::view` projections, and the requesting player's legal actions leave the server as game data. Never serialize a `Match`/`Session` as a live response or expose seeds/stock order/other hands. Skyjo's stock draw is visible only to its active player. Tempel des Schreckens exposes the viewer's role and remaining card counts, but no hidden card positions, including their own, or other roles.
- Replays intentionally contain all hidden information. Only archive a completed game, and only let room members download it. Replay format 1 fixes rules, RNG algorithm, deck order, and action semantics; incompatible changes require a new version and compatibility strategy.
- A room mutex serializes joins and commands. Make a candidate copy, persist it (and any completed replay) in a transaction, then replace memory and notify subscribers. Failed writes must not publish a state change.
- Bearer tokens identify Members, not game Players, nicknames, or room codes. WebSockets authenticate via the first frame, never a URL query parameter. Never log tokens. Members have stable IDs; each game's frozen ordered member roster defines its Player indices. Host authority is independent of seating, including on the bench. Benched members receive no game view or legal actions.
- Clients submit their latest observed room revision. Nested game messages identify the game and may use an older revision between its explicit start revision and the current room revision; revalidate every move. Joins advance room revisions independently of games. Setup/start/abort commands require the exact revision. Only a command response, not another member's broadcast, clears client pending state. Reconnects send a fresh snapshot; never automatically resend an unacknowledged move.
- Rooms need no selected game. New Members join the bench at any time. Only the host configures games and seating, which lock during unfinished games. Starts freeze the roster/configuration; aborts clear the game without archiving unfinished state. Completed archives retain their roster and are accessible to every room Member.
- Edit database initialization in `schema.sql` directly; compatibility migrations are not required. The storage model assumes one server process.

## Changing a game

Follow `docs/adding-a-game.md` for the end-to-end checklist and Tic-Tac-Toe example.

Each game's `rules.md`, tests, and implementation must always agree. Any change to `rules.md` must prompt corresponding test updates and, when behavior changes, game-logic updates. Keep these changes together in the same feature or fix commit; do not document new rules without testing and implementing them.

Implement `Game` with its action, event, and view types; reuse `Event` or implement `GameEvent` for a custom event type. Add dispatch in `arena.rs`, board rendering in `GameBoard.svelte`, and a name in `types.ts`. Update the game's `rules.md`. Use unique `#[schema(as = ...)]` names for game-local `Action`/`View` types to avoid OpenAPI collisions. Do not force every game to use all shared utilities.

Add rule-edge tests in the game module, seeded simulations/replay checks in `arena-core/tests`, and server/browser coverage where behavior crosses boundaries. Test malformed indices, rejected-action immutability, hidden information, exhausted decks, termination, and supported player counts.

## Verification

Run Rust format, Clippy with warnings denied, and workspace tests. Run frontend `check`, `build`, and Playwright (commands in README). Protocol changes must regenerate both checked-in files:

```sh
cargo run -q -p arena-server -- --openapi > openapi.json
npm --prefix frontend run api:generate
```

Use `npm --prefix frontend run format` for frontend formatting. Prefer established dependencies to homegrown infrastructure. Avoid adding layers solely for a hypothetical future feature. Keep text and styling restrained, with accessible labels and keyboard controls. Make atomic conventional commits; do not discard unrelated changes.
