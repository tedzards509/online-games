# Protocol

[`../openapi.json`](../openapi.json) is generated from Rust types and route annotations. `/api/openapi.json` serves the same specification. WebSocket message schemas are included as `ClientMessage` and `ServerMessage`; OpenAPI describes the upgrade, not the ongoing stream.

## Enter a room

`POST /api/rooms` with `{"nickname":"Alice","game":"mancala"}` creates a room. `POST /api/rooms/ABC234/join` with `{"nickname":"Bob"}` joins it. Both return `{"code":"ABC234","player":0,"token":"…"}` (the joining seat differs). Codes are case-insensitive. Nicknames are unique within a room, trimmed, 1–24 characters, with no control characters.

Keep the returned token private. For HTTP reads use `Authorization: Bearer <token>`:

- `GET /api/rooms/{code}`: your personalized snapshot.
- `GET /api/rooms/{code}/replays/{id}`: a completed replay; IDs appear in snapshots.
- `GET /api/health`: 204 when startup, database migration, and room loading have succeeded.

No tokens in URLs, no cookies, and no account identity. Joining is closed once a game has started; existing seats can reconnect at any time. The host is seat 0.

## Live channel

Connect to `/api/rooms/{code}/ws`, then send within five seconds:

```json
{"type":"authenticate","token":"…"}
```

The server replies with `{"type":"snapshot","room":{…}}`. A snapshot includes the room revision, players, your seat, current turn, outcome, a game-specific `view`, your `legal_actions`, and archived replay IDs. Before starting, `view` and `turn` are null. A turn contains `players` (eligible seat indices) and `number` (the ruleset's counter). Only eligible seats receive legal actions, personalized per seat. Every accepted change notifies all room sockets; each receives a fresh, independently projected snapshot, not the private engine state or raw event log.

Start or rematch (host only; at least two players, no running game):

```json
{"type":"start","revision":2}
```

Use an action from your latest snapshot's `legal_actions`:

```json
{"type":"move","revision":3,"action":{"game":"mancala","action":{"pit":0}}}
```

Every accepted join/start/move increments the revision. A stale revision, wrong turn, invalid action, or finished game is rejected without mutation. An authenticated command error returns `{"type":"error","error":"…"}` followed by a fresh snapshot. A rejected initial authentication closes the connection. Browser origins must match `PUBLIC_ORIGIN`; native clients can omit Origin.

When multiple eligible seats submit for the same revision, the first valid command committed under the room lock wins. Other submissions for that revision are rejected as stale, even if their seats remain eligible. Failed validation or persistence does not consume the revision. This is server processing order, not client timestamp or seat priority.

The server pings every 20 seconds and disconnects clients that miss the next heartbeat. Reconnect, authenticate again, and replace local state with the new snapshot. Do not resend old moves automatically: a committed move may have succeeded before the connection dropped.

## Replays and simulations

The server archives completed games transactionally. A replay contains a format version, game kind, player count, 32-byte seed, and accepted actions with resulting events. It intentionally reveals hidden cards, so running games have no downloadable replay. `Replay::simulate` recreates the game, validates every action, and checks the event history. The core also exposes `Game::advance` for fast, log-free simulation of actions drawn from `legal_actions`.
