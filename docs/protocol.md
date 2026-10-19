# Protocol

[`../openapi.json`](../openapi.json) is generated from Rust types and route annotations. `/api/openapi.json` serves the same specification. WebSocket message schemas are included as `ClientMessage` and `ServerMessage`; OpenAPI describes the upgrade, not the ongoing stream.

## Enter a room

`POST /api/rooms` with `{"nickname":"Alice","game":"mancala"}` creates a room. `POST /api/rooms/ABC234/join` with `{"nickname":"Bob"}` joins it. Both return `{"code":"ABC234","player":0,"token":"…"}` (the joining seat differs). Codes are case-insensitive. Nicknames are unique within a room, trimmed, 1–24 characters, with no control characters.

Keep the returned token private. For HTTP reads use `Authorization: Bearer <token>`:

- `GET /api/rooms/{code}`: your personalized snapshot.
- `GET /api/rooms/{code}/replays/{id}`: a completed replay; IDs appear in snapshots.
- `GET /api/health`: 204 when startup, database migration, and room loading have succeeded.

### Host-recorded results

The Crew supports 3–5 players with empty `rules: {}` and no missions. Its snapshots expose personalized `can_finish`: true only for the host while the game is running. The host sends `{"type":"finish","revision":4,"won":true}` (or `false`) as a room command, separate from `move`. It accepts older observed revisions within the current game like queued moves, but rejects future or previous-game revisions. The result lists all seats for a win or no seats for a loss, without scores. Other games reject manual finish and retain automatic results.

Finish commits and archives the replay transactionally, even during a partial trick. Internally the replay contains a game-tagged finish action; sending that internal action via `move` is rejected. It is excluded from snapshot `legal_actions` and turn eligibility. The board's `next_player` identifies the next card-playing seat; other eligible seats may communicate between tricks. Once no complete trick remains, the room waits for a host result. No changes to room membership or game selection are introduced here.

No tokens in URLs, no cookies, and no account identity. Joining is closed once a game has started; existing seats can reconnect at any time. The host is seat 0.

The host selects immutable options before room creation. Supply `config` as `{"game":"crazy_eights","rules":{"ace_are_you_strong":true,"jack_skip":true,"ten_reverse":true,"two_draw_two":true}}` alongside the matching `game` and nickname. Eights are always wild; `eight_wild` is no longer accepted. Shithead uses `{"game":"shithead","rules":{"four_card_bombs":false}}`; other games use empty `rules: {}`. Omitted configuration selects current defaults. Mismatched game tags, unknown rule fields, and nonboolean rule values are rejected. Every HTTP/WebSocket snapshot includes public `config`, even before starting and after reconnecting. Rematches retain it; there is no configuration update command.

## Live channel

Connect to `/api/rooms/{code}/ws`, then send within five seconds:

```json
{"type":"authenticate","token":"…"}
```

The server replies with `{"type":"snapshot","room":{…},"acknowledged":false}`. A snapshot includes the room revision, players, your seat, current turn, outcome, a game-specific `view`, your `legal_actions`, and archived replay IDs. Before starting, `view` and `turn` are null. A turn contains `players` (eligible seat indices) and `number` (the ruleset's counter). Only eligible seats receive legal actions, personalized per seat. Every accepted change notifies all room sockets; each receives a fresh, independently projected snapshot, not the private engine state or raw event log. Successful command responses to the submitting socket set `acknowledged: true`; broadcasts and initial/error-refresh snapshots set it to false. Broadcasts may coalesce intermediate revisions and must not clear an outstanding command's pending state.

Start or rematch (host only; at least two players, no running game):

```json
{"type":"start","revision":2}
```

Use an action from your latest snapshot's `legal_actions`:

```json
{"type":"move","revision":3,"action":{"game":"mancala","action":{"pit":0}}}
```

Every accepted join/start/move increments the revision. Starts require the exact current revision. Moves accept the latest observed revision or an older one from the same game, but reject future revisions or revisions before the current start/rematch. Wrong-turn, illegal, and post-game moves are rejected without mutation. An authenticated command error returns `{"type":"error","error":"…"}` followed by a fresh snapshot. Validation or storage failures leave state unchanged. A rejected initial authentication closes the connection. Browser origins must match `PUBLIC_ORIGIN`; native clients can omit Origin.

Incoming moves are queued FIFO by the room mutex. Each is validated against the state resulting from earlier successful commits; every still-legal move executes, even if several commands reference the same revision. Only moves made invalid by that updated state fail game validation. Invalid commands and failed writes do not consume a revision or stop later queued moves. Queue order is server lock-acquisition order, not client timestamp or seat priority. This is not deduplication: sending a move twice can execute it twice if both are legal, so clients must not resend automatically.

The server pings every 20 seconds and disconnects clients that miss the next heartbeat. Reconnect, authenticate again, and replace local state with the new snapshot. Do not resend old moves automatically: a committed move may have succeeded before the connection dropped.

## Replays and simulations

The server archives completed games transactionally. A replay contains a format version, game kind, player count, 32-byte seed, and accepted actions with resulting events. `GameKind::replay_version` selects the supported rules revision; newer revisions include explicit `config`. Older incompatible replays require the corresponding previous implementation (see [compatibility](design.md)). It intentionally reveals hidden cards, so running games have no downloadable replay. `Replay::simulate` recreates the game, validates every action, and checks the event history. The core also exposes `Game::advance` for fast, log-free simulation of actions drawn from `legal_actions`.
