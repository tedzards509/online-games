# Add a game

Use `crates/arena-core/src/games/tic_tac_toe/` and `frontend/src/components/TicTacToe.svelte` as the smallest complete example.

1. **Fix the rules.** Choose a stable snake-case game ID. Document player counts, turn order, special cases, and win/draw conditions in `crates/arena-core/src/games/<id>/rules.md`, and link it from the README's game rules section.
2. **Implement the core.** Add `crates/arena-core/src/games/<id>/mod.rs` and export it from `games/mod.rs`. Implement `Game` with action, event, and view types. State needs `Clone`, `Serialize`, and `Deserialize`; actions also need `PartialEq`; actions/views need `ToSchema` with unique names such as `TicTacToeAction`.
   - `Config`: define a serializable, comparable, defaultable `GameConfig` struct, or use `game::GameConfig` for no options. Register it in runtime `Config` and `GameKind::default_config`, plus `Match::config` and `with_config`.
   - `with_config`: validate player count and apply setup options; use only the supplied seed for randomness (ignore it for deterministic games). `new` uses defaults. Preserve original defaults for version-1 replays.
   - `legal_actions(player)`: list that seat's valid commands; return none for ineligible seats or after completion.
    - `advance(player, action)`: apply a legal command for its submitting seat, update turns/outcomes, and emit effects. `Session::act` validates first and appends the final event. Set `turn.players` to every seat with a legal command (including communication or host-only decisions), not just the next card-playing seat; expose that seat separately when needed. `Turn::advance(count)` is only for singleton lists and panics otherwise. Queued submissions are revalidated against the latest committed state; older same-game revisions remain valid.
   - `view`: expose only information that this viewer may see.
   Reuse `Event` for arena integration. Custom `GameEvent` types work with generic sessions; integrating them into `Match` also requires adapting its replay event representation. Cards and decks are optional.
3. **Register it.** In `arena.rs`, extend `GameKind`, `max_players`, `Action`, `View`, `Match`, `dispatch!`, and every ruleset match arm. Add the ruleset to `examples/simulate.rs`. Existing replay-version-1 rules must remain unchanged; adding a new ID alone needs no version bump. Rooms and SQLite need no game-specific code or migration.
4. **Wire the board.** Regenerate the protocol, then add a board in `frontend/src/components/`, dispatch it in `GameBoard.svelte`, and add its name in `lib/types.ts`:
   ```sh
   cargo run -q -p arena-server -- --openapi > openapi.json
   npm --prefix frontend run api:generate
   ```
    Render the projected view and submit actions from `legal_actions`. For configurable games, add pre-creation controls in `App.svelte` and a public rule summary in `GameConfig.svelte`; send the game-tagged `Config` when creating the room. Rooms retain and broadcast it without game-specific server code. Let the existing connection handle authentication, revisions, pending state, and reconnects; use accessible buttons and labels.
5. **Test the integration.** Add rule-edge tests beside the ruleset: wins, draws, invalid indices, wrong turns, unchanged state/log on rejection, and post-game moves. Cover hidden information and exhausted decks where applicable. Include the game in the core simulation/player-count tables, server WebSocket/replay and schema tests, and browser tests. Tic-Tac-Toe also checks every reachable position against its expected legal actions and outcome.
6. **Submit.** Run all README checks and format the frontend with `npm --prefix frontend run format`. Check `git diff --check`; include source, tests, rules, and both generated API files in focused conventional commits. Open a pull request summarizing the rules and checks. Do not include local IDE files or change unrelated work.
