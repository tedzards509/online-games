# Design notes

## Game model

The contract is defined in `crates/arena-core/src/game.rs`; runtime dispatch and replay types live in `arena.rs`.

| Concept | Definition |
| --- | --- |
| `Game` / state | A ruleset's concrete type is its complete authoritative state: board, hands, stock, phases, turn, outcome, and any RNG state. There is no separate state trait. |
| `Player` | A zero-based seat index (`usize`), not a nickname or authentication token. |
| `Seed` | Explicit `[u8; 32]` input to setup. The same rules, player count, seed, and actions must reproduce the same states and events. |
| `Game::Config` / `GameConfig` | Serializable, immutable setup options defined by each game; runtime `Config` tags the game-owned struct. Options are public, unlike private state and seeds. |
| `Action` | A proposed game-specific command, not an effect or state snapshot. The submitting player is passed separately. |
| Legal actions | `legal_actions(player)` enumerates that seat's valid commands in the current phase; empty for ineligible seats or after completion. |
| `Event` | An ordered effect of an accepted action, e.g. placement, capture, or completion. Events describe transitions; they are not commands or independently applied state updates. `GameEvent::finished` constructs the terminal event. |
| `Turn` | Eligible `players: Vec<Player>` and ruleset-managed `number`. The list is not a queue or priority order. The ruleset owns extra turns, response windows, and multi-action phases. `Turn::advance(count)` rotates a singleton list and increments the counter; it panics for any other list length. Neither field is a room revision. |
| `Outcome` | `None` during play; otherwise winning seats and optional scores in seat order. Draws and shared wins follow the ruleset's conventions. |
| `View` | A viewer-specific projection of state. Only permitted information is exposed; full state and seeds remain private. |
| `Session<G>` / `Record<A, E>` | A validation wrapper holding current state, setup inputs, and history. Each record contains the submitting player, accepted action, and resulting events. |
| `Match` / `Replay` | Runtime ruleset dispatch / versioned setup and action history. Replay validation recreates setup, executes actions, and compares regenerated records, including events. |

### Transition contract

1. `Game::with_config(players, seed, config)` validates player count and constructs deterministic setup. `Game::Config` is a game-owned serializable `GameConfig` struct (the shared empty struct suffices for games without options). `new` supplies its defaults. `Session` retains immutable options alongside seed and history; runtime `Config` tags each struct with its game ID.
2. `Session::act(player, action)` rejects completed games, ineligible players, and actions outside `legal_actions(player)` before any mutation or logging.
3. `advance(player, action)` mutates state, resolves rule effects, updates phases/turn/outcome, and returns events. It assumes a legal action; simulations may call it directly with actions selected from `legal_actions(player)`.
4. The session appends a terminal event if the game ended, then records the action and events. It stores no per-event state snapshots.

Rules are synchronous and deterministic: no clocks, OS randomness, I/O, or transport dependencies. See [Add a game](adding-a-game.md) for implementation and integration steps.

Multiple seats may submit against the same observed room revision. Tokio's room mutex is the FIFO command queue; each move is checked against the latest successfully persisted state when it reaches the head. Commit every legal move, including moves whose observed revision predates queued commits. Reject out-of-turn, illegal, or post-game moves without mutation; a rejected move does not block later commands. No timestamps, batching window, background queue worker, or automatic retries are used.

Clients send their latest observed revision. Game messages identify a game and may reference older revisions within it, but never a future revision or one before its explicit start revision. Joins can advance the room revision independently of game actions. Setup/start/abort commands require the exact revision. Each commit persists before memory replacement and notification; failed writes do not consume a revision. Broadcast snapshots carry `acknowledged: false`; the submitting socket receives `acknowledged: true` after success. Only that response (or an error/disconnection) releases pending UI state. Snapshots may coalesce intermediate commits; accepted game history preserves action order.

## Review of `../spiele`

- `mancala/src/mancala_game.rs` and `mancala/mancala.md` are empty. Its entrypoint only prints a greeting. There is no Mancala rules implementation to reuse.
- Tic-Tac-Toe cleanly distinguishes commands, events, and state. Its small enums and pure winner checks are useful patterns. Keeping an initial setup and a replayable history is the right direction.
- Its board indexes unchecked coordinates; malformed input can panic. Turn order belongs to the CLI rather than the game, arbitrary player markers are accepted, and commands remain accepted after completion. An action is logged before its derived placement is validated, so an occupied cell leaves a partial history. Full state snapshots per event are unnecessary for these games.
- Skyjo has useful seeded randomization, explicit card visibility, and a correct 150-card distribution. However, moves and event application are unfinished. `distribute_cards` returns layouts without assigning them; row/column construction is reversed; one draw path bypasses distribution bookkeeping; replacement pops the discard before validating the target. Iterating a `HashMap` before a seeded shuffle also prevents reproducible deals across processes.
- The existing `GameState: Debug + Clone` marker imposes no behavioral contract.

The framework uses deterministic setup, typed actions/events, per-player observations, and replayable transitions. Tic-Tac-Toe is the worked example for the [new-game checklist](adding-a-game.md).

## Boundaries

`arena-core` owns rules, random state, turn order, legal actions, observations, and replay validation. `Game` can be used directly for simulations; `Session` validates before mutation and records accepted actions and resulting events. It keeps one current state, not a state copy per event. `Deck<C>` is independent of the standard playing card type; Skyjo uses numeric cards, and Mancala uses neither.

Actions, events, and observations are associated types of `Game`. The included games share an `Event` enum, but another ruleset can provide its own `GameEvent` and still use `Session` and `Record<A, E>`.

The Crew uses its own colored cards with `Deck<Card>`. Its next card-playing seat is separate from eligibility: between tricks other players may communicate. Communications retain their card/declaration after play; private hands are never projected to other seats. Without missions, exhausting playable tricks waits for the host's result decision. The Crew uses replay version 1.

Tempel des Schreckens keeps roles private and projects only each viewer's unrevealed card counts, never their positions. Public rows contain face-up card values and hidden placeholders. The host chooses the initial key; simultaneous readiness precedes each reveal phase. The key passes to the revealed card's owner and persists across seeded redeals of unrevealed cards. Revealed-pile counts persist for all four rounds; outcomes list the seats with the winning role.

Host controls are nested game messages routed by `crates/arena-server/src/game.rs`, separate from room lifecycle, settings, and normal player moves. The Crew's `finish` records its shared result; Tempel's `choose_key` selects its initial key holder. Personalized `can_finish`/`can_choose_key` capabilities depend on host membership and game phase, not seating. The server authorizes the host, then records the external decision through core player 0 for deterministic replay. Normal moves cannot submit these internal actions. Host controls use the same game identity, FIFO validation, and transactional persistence as moves.

`arena-server` owns anonymous member identities, rooms, seating, persistence, and transport. A token identifies a stable room member. The creator is the host independently of seating. An ordered subset of members becomes a game's players; omitted members are benched. New members join the bench at any time. Membership IDs and game-player indices are distinct. The game roster is frozen at start and used to translate incoming moves and build personalized views. Benched members receive no game view or legal actions; spectating requires explicit safe projections and board support.

Rooms can exist without a selected or running game. The host edits public configuration and seating through WebSocket commands; all members receive accepted changes live. Setup is locked while a game is unfinished. Start validates the player count and captures configuration, roster, a fresh game ID, and start revision. A completed snapshot retains its own setup while the host configures the next game; starting with unchanged settings is a rematch. Abort clears the current game while retaining room setup, membership, and completed archives. Aborted games have no result or replay.

Private engine state is persisted but never sent as a live observation. Completion archives a replay exactly once in the same transaction as the room update. Archive summaries preserve game configuration and ordered member identities. All room members, including later arrivals, may download completed replays, which intentionally expose hidden information. Schema initialization uses `schema.sql`; persisted room JSON requires the current data model, without compatibility migrations.

## Replay and save compatibility

`GameKind::replay_version` selects the supported rules revision. Validation checks the game tag, version, actions, and resulting events. Each version fixes the rules, RNG algorithm, deck order, and action semantics.

| Games | Supported replay version | Configuration | Saved-session requirement |
| --- | --- | --- | --- |
| Crazy Eights, Mancala, Shithead, Skyjo | 4 | Explicit, matching game tag | `rules_version: 4` |
| Tic-Tac-Toe, The Crew, Tempel des Schreckens | 1 | Omitted; ruleset defaults | No rules-version marker |

Unsupported replay versions and mismatched saved-session markers are rejected. Stored rooms require the current data model; use a fresh database for incompatible stored data. Database compatibility migrations are not provided.
