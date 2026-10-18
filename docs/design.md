# Design notes

## Review of `../spiele`

- `mancala/src/mancala_game.rs` and `mancala/mancala.md` are empty. Its entrypoint only prints a greeting. There is no Mancala rules implementation to reuse.
- Tic-Tac-Toe cleanly distinguishes commands, events, and state. Its small enums and pure winner checks are useful patterns. Keeping an initial setup and a replayable history is the right direction.
- Its board indexes unchecked coordinates; malformed input can panic. Turn order belongs to the CLI rather than the game, arbitrary player markers are accepted, and commands remain accepted after completion. An action is logged before its derived placement is validated, so an occupied cell leaves a partial history. Full state snapshots per event are unnecessary for these games.
- Skyjo has useful seeded randomization, explicit card visibility, and a correct 150-card distribution. However, moves and event application are unfinished. `distribute_cards` returns layouts without assigning them; row/column construction is reversed; one draw path bypasses distribution bookkeeping; replacement pops the discard before validating the target. Iterating a `HashMap` before a seeded shuffle also prevents reproducible deals across processes.
- The existing `GameState: Debug + Clone` marker imposes no behavioral contract.

Ported ideas, not source: deterministic setup, typed actions/events, per-player observations, and replayable transitions. Tic-Tac-Toe itself is not included in the initial game list.

## Boundaries

`arena-core` owns rules, random state, turn order, legal actions, observations, and replay validation. `Game` can be used directly for simulations; `Session` validates before mutation and records accepted actions and resulting events. It keeps one current state, not a state copy per event. `Deck<C>` is independent of the standard playing card type; Skyjo uses numeric cards, and Mancala uses neither.

Actions, events, and observations are associated types of `Game`. The included games share an `Event` enum, but another ruleset can provide its own `GameEvent` and still use `Session` and `Record<A, E>`.

`arena-server` owns anonymous identities, rooms, persistence, and transport. Private engine state is persisted but never sent as a live observation. Replays include the seed and therefore reveal every card; downloads are restricted to finished games and room members.

Replay format version 1 fixes the initial rules and RNG/dealing behavior. Rules or RNG changes that alter historical results need a new replay version, not silent reinterpretation of existing files.
