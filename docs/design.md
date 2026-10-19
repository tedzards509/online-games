# Design notes

## Game model

The contract is defined in `crates/arena-core/src/game.rs`; runtime dispatch and replay types live in `arena.rs`.

| Concept | Definition |
| --- | --- |
| `Game` / state | A ruleset's concrete type is its complete authoritative state: board, hands, stock, phases, turn, outcome, and any RNG state. There is no separate state trait. |
| `Player` | A zero-based seat index (`usize`), not a nickname or authentication token. |
| `Seed` | Explicit `[u8; 32]` input to setup. The same rules, player count, seed, and actions must reproduce the same states and events. |
| `Action` | A proposed game-specific command, not an effect or state snapshot. The submitting player is passed separately. |
| Legal actions | `legal_actions()` enumerates exactly the active player's valid commands in the current phase; empty after completion. |
| `Event` | An ordered effect of an accepted action, e.g. placement, capture, or completion. Events describe transitions; they are not commands or independently applied state updates. `GameEvent::finished` constructs the terminal event. |
| `Turn` | Active `player` and ruleset-managed `number`. The ruleset owns extra turns, skipped seats, and multi-action phases; one action need not end a turn. Neither field is a room revision. |
| `Outcome` | `None` during play; otherwise winning seats and optional scores in seat order. Draws and shared wins follow the ruleset's conventions. |
| `View` | A viewer-specific projection of state. Only permitted information is exposed; full state and seeds remain private. |
| `Session<G>` / `Record<A, E>` | A validation wrapper holding current state, setup inputs, and history. Each record contains the submitting player, accepted action, and resulting events. |
| `Match` / `Replay` | Runtime ruleset dispatch / versioned setup and action history. Replay validation recreates setup, executes actions, and compares regenerated records, including events. |

### Transition contract

1. `Game::new(players, seed)` validates player count and constructs deterministic setup.
2. `Session::act(player, action)` rejects completed games, inactive players, and actions outside `legal_actions()` before any mutation or logging.
3. `advance(action)` mutates state, resolves rule effects, updates phases/turn/outcome, and returns events. It assumes a legal action; simulations may call it directly with actions selected from `legal_actions()`.
4. The session appends a terminal event if the game ended, then records the action and events. It stores no per-event state snapshots.

Rules are synchronous and deterministic: no clocks, OS randomness, I/O, or transport dependencies. See [Add a game](adding-a-game.md) for implementation and integration steps.

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

`arena-server` owns anonymous identities, rooms, persistence, and transport. Private engine state is persisted but never sent as a live observation. Replays include the seed and therefore reveal every card; downloads are restricted to finished games and room members.

Replay format version 1 fixes the initial rules and RNG/dealing behavior. Rules or RNG changes that alter historical results need a new replay version, not silent reinterpretation of existing files.
