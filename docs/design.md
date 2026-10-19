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

Multiple seats may submit against the same observed room revision. Tokio's room mutex is the FIFO command queue; each move is checked against the latest successfully persisted state when it reaches the head. Commit every move that is still legal, even when earlier moves advanced the revision. Reject moves that are now out of turn, illegal, or post-game without mutation; a rejected move does not block later commands. No timestamps, batching window, background queue worker, or automatic retries are used.

Clients send their latest observed revision. Moves may reference older revisions within the current game, but never a future revision or one before its start/rematch. The start revision is the current room revision minus the game's accepted-action count, since joins are closed during play. Starts require the exact current revision to prevent stale rematches. Each commit persists independently before memory replacement and notification; failed writes do not consume a revision. Broadcast snapshots carry `acknowledged: false`; the submitting socket also receives a current snapshot with `acknowledged: true` after its command succeeds. Only that response (or an error/disconnection) releases its pending UI state. Snapshots may coalesce intermediate commits; the accepted-action log preserves their order.

## Review of `../spiele`

- `mancala/src/mancala_game.rs` and `mancala/mancala.md` are empty. Its entrypoint only prints a greeting. There is no Mancala rules implementation to reuse.
- Tic-Tac-Toe cleanly distinguishes commands, events, and state. Its small enums and pure winner checks are useful patterns. Keeping an initial setup and a replayable history is the right direction.
- Its board indexes unchecked coordinates; malformed input can panic. Turn order belongs to the CLI rather than the game, arbitrary player markers are accepted, and commands remain accepted after completion. An action is logged before its derived placement is validated, so an occupied cell leaves a partial history. Full state snapshots per event are unnecessary for these games.
- Skyjo has useful seeded randomization, explicit card visibility, and a correct 150-card distribution. However, moves and event application are unfinished. `distribute_cards` returns layouts without assigning them; row/column construction is reversed; one draw path bypasses distribution bookkeeping; replacement pops the discard before validating the target. Iterating a `HashMap` before a seeded shuffle also prevents reproducible deals across processes.
- The existing `GameState: Debug + Clone` marker imposes no behavioral contract.

Ported ideas, not source: deterministic setup, typed actions/events, per-player observations, and replayable transitions. Tic-Tac-Toe was subsequently added as the worked example for the [new-game checklist](adding-a-game.md).

## Boundaries

`arena-core` owns rules, random state, turn order, legal actions, observations, and replay validation. `Game` can be used directly for simulations; `Session` validates before mutation and records accepted actions and resulting events. It keeps one current state, not a state copy per event. `Deck<C>` is independent of the standard playing card type; Skyjo uses numeric cards, and Mancala uses neither.

Actions, events, and observations are associated types of `Game`. The included games share an `Event` enum, but another ruleset can provide its own `GameEvent` and still use `Session` and `Record<A, E>`.

The Crew uses its own colored cards with `Deck<Card>`. Its next card-playing seat is separate from eligibility: between tricks other players may communicate. Communications retain their original card/declaration after play; private hands are never projected to other seats. Without missions, exhausting playable tricks waits for the host rather than creating an automatic outcome. The new game starts at replay version 1; existing rules revisions are unchanged.

The host's `finish` room command records a shared result only for games that support it (currently The Crew). The room exposes a personalized `can_finish` capability, separate from playable actions and turn eligibility. It translates the command into an internal game finish action for deterministic replay; clients cannot submit that action via `move`. Finish uses the same FIFO validation, current-game revision boundary, atomic persistence, and replay archival as moves. This keeps room lifecycle controls outside board rendering without introducing a multi-game room model yet.

`arena-server` owns anonymous identities, rooms, persistence, and transport. Private engine state is persisted but never sent as a live observation. Replays include the seed and therefore reveal every card; downloads are restricted to finished games and room members.

The host chooses configuration before creating a room. The server checks its game tag, persists it with the room, and includes it in every personalized snapshot. Starts and rematches use those same options. There is no in-room configuration mutation; existing rooms without options use original defaults.

Replay format version 1 fixes the initial rules and RNG/dealing behavior and omits configuration. Version 2 records nondefault options explicitly. Version 3 introduced voluntary draws, nonchaining ace responses, and permanently wild eights in Crazy Eights. Version 4 uses explicit configuration: no pass-based draw in Crazy Eights; tied scores as a draw in Mancala; simultaneous face-up selection, transparent threes, restrictive sevens, and configurable bombs in Shithead; simultaneous reveals during Skyjo setup. Tic-Tac-Toe retains version 1. `GameKind::replay_version` selects the supported revision. Validation checks the game tag, version, actions, and resulting events.

Older versions of revised games are deliberately rejected rather than interpreted using changed rules. Saved sessions require a matching `rules_version` marker. Retain revision `57305f1` for their previous rooms/replays and `b0fb0e2` for earlier Crazy Eights rules. Back up databases and use a separate database for revised rules; no old data is automatically rewritten or removed. Tic-Tac-Toe saved sessions remain compatible. Future incompatible changes likewise require an explicit version and compatibility strategy.
