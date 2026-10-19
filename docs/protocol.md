# Protocol

[`../openapi.json`](../openapi.json) is generated from Rust types and route annotations. WebSocket schemas are included as `ClientMessage`, `GameMessage`, and `ServerMessage`; OpenAPI describes the upgrade, not the ongoing stream.

## Membership

`POST /api/rooms` with `{"nickname":"Alice"}` creates an unconfigured room. `POST /api/rooms/ABC234/join` with `{"nickname":"Bob"}` joins it, including during play. Both return `{"code":"ABC234","member":0,"token":"…"}` with the member's stable room index. Codes are case-insensitive. Nicknames are unique within a room, trimmed, 1–24 characters, with no control characters.

Members are room identities; players are zero-based seats within one game. Tokens authenticate members, not players. The creator is the host, independently of seating, and may remain on the bench. New members join the bench. Only the host may arrange seating; removal and host transfer are not supported. Room capacity is independent of game player limits.

Use `Authorization: Bearer <token>` for HTTP reads:

- `GET /api/rooms/{code}`: personalized room snapshot.
- `GET /api/rooms/{code}/replays/{id}`: completed replay, available to every room member, including benched members and later arrivals.
- `GET /api/health`: 204 after schema initialization and room loading succeed.

Tokens are never placed in URLs. No cookies or accounts are used.

## Room channel

Connect to `/api/rooms/{code}/ws`, then send within five seconds:

```json
{"type":"authenticate","token":"…"}
```

The server sends `{"type":"snapshot","room":{…},"acknowledged":false}`. Room snapshots contain:

- `members`: stable IDs and nicknames; `host` and `you` identify members.
- `config`: selected game and public settings, or null before selection.
- `seating`: member IDs in the next game's player order; omitted members are benched.
- `game`: current game's snapshot, or null. Its `id`, `config`, and ordered `players` belong to that game. Its `you` is the viewer's player index, or null for a benched member. `turn.players` and outcome indices refer to this roster, not room membership or next-game seating.
- `replays`: completed-game summaries with ID, configuration, and ordered member roster.
- `revision`: increases for every accepted room change or game message, including joins.

Host-only setup commands require the exact current revision:

```json
{"type":"configure","revision":2,"config":{"game":"crazy_eights","rules":{"ace_are_you_strong":true,"jack_skip":false,"ten_reverse":false,"two_draw_two":true}}}
{"type":"seat","revision":3,"members":[1,0]}
{"type":"start","revision":4}
```

`configure` selects a game and its complete settings atomically; null clears the selection. Shithead uses `{"game":"shithead","rules":{"four_card_bombs":false}}`; other games use empty `rules: {}`. Unknown fields and invalid rule types are rejected. Every accepted setting change is broadcast live. Configuration and seating are locked while the current game is unfinished. Selecting a game does not implicitly seat or bench anyone; `start` validates its player count.

`seat` supplies the complete ordered list of players for the next game. Each ID must identify a distinct member. Everyone absent is on the bench. Starting freezes this roster and configuration into a fresh game with its own ID and explicit start revision. Reordering never changes member identity, authentication, or host authority.

Completed games remain visible. The host may edit next-game settings/seating and start another game; the completed snapshot and replay retain their original roster. Starting again with unchanged setup is a rematch using the same path.

The host may clear the current game:

```json
{"type":"abort","revision":5,"game_id":"…"}
```

Abort requires the exact revision and matching game ID. It removes the current game without assigning a result or archiving an unfinished replay. Membership, settings, seating, and completed replays remain. For a completed game it returns to setup without removing its archive.

## Game messages

Use a legal action from your personalized game snapshot:

```json
{"type":"game","revision":5,"game_id":"…","message":{"type":"move","action":{"game":"mancala","action":{"pit":0}}}}
```

The server maps the authenticated member to their immutable player index and routes the action to that game. The game tag must match. Benched members cannot move and receive no `view` or legal actions; spectator projections are not implemented. Player observations never expose private engine state or seeds.

Game messages require the matching game ID and an observed revision between that game's start revision and the current room revision. Older same-game revisions remain valid even across unrelated joins. Future, pre-start, aborted-game, and previous-game messages are rejected. The room mutex queues commands FIFO; each action is revalidated against the latest successfully persisted state. Every still-legal move executes. A rejected action or failed write does not consume a revision or block later commands. This is not deduplication: do not automatically resend an unacknowledged move.

### Host game controls

The Crew (3–5 players, without missions) waits for the group's shared win/loss decision. The host sends:

```json
{"type":"game","revision":5,"game_id":"…","message":{"type":"finish","won":true}}
```

`game.can_finish` is true only for the host while The Crew is unfinished, even if the host is benched. Finish can occur during a trick; a win lists all players, a loss lists none, and neither assigns scores. It is a game lifecycle decision, not a setting or playable action. Other games reject it. Internally, the core records it deterministically through player 0; that player's membership does not confer host authority. Submitting the internal finish through `move` is rejected.

Tempel des Schreckens (3–10 players) exposes `game.can_choose_key` to the host during initial key selection:

```json
{"type":"game","revision":5,"game_id":"…","message":{"type":"choose_key","player":1}}
```

The target is a game-player index. The host need not be seated. The core records the choice through player 0; clients cannot submit it as a normal move. Each round then uses simultaneous `ready` moves and key-holder `reveal` moves. Players see only their own role and remaining counts, with nulls for every hidden position, including their own row. Key ownership and revealed-pile counts persist across redeals.

## Responses, persistence, and reconnects

Every accepted change persists before memory replacement and notification. Completion and replay archival share that transaction. Snapshots may coalesce intermediate revisions; accepted game history retains action order.

Successful command responses set `acknowledged: true` only for the submitting socket. Broadcasts, initial snapshots, and error refreshes set it to false. Broadcasts must not clear pending client commands. An authenticated command error sends `{"type":"error","error":"…"}` followed by a current snapshot; failed validation/storage leaves state unchanged. Initial authentication failure closes the connection.

Origins must match `PUBLIC_ORIGIN`; native clients may omit Origin. The server pings every 20 seconds and disconnects members who miss the next heartbeat. Reconnect and authenticate to receive a fresh personalized snapshot; never resend old commands automatically.

Schema initialization uses `crates/arena-server/schema.sql`; stored room JSON must match the current implementation. Use a fresh database for incompatible stored data.

## Replays

Only completed games are archived; aborts do not export hidden state. Replay summaries retain the game configuration and member roster. Downloads contain the core replay: rules version, game kind, player count, seed, and accepted actions/events. They intentionally reveal all hidden information. `Replay::simulate` reconstructs and validates the history. All authenticated room members may download completed replays.
