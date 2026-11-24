# Game state visible to players

This document defines the live game information a seated player receives in their personalized room snapshot. It describes
the server's current game snapshot and each game's `view`; it is not a description of replay data. Benched room members
receive no game snapshot, game view, or legal actions. The exact wire types are in `crates/arena-server/src/protocol.rs`
and the per-game view types are in `crates/arena-core/src/games/<id>/mod.rs`.

## JSON Schema

The canonical, complete schema is generated in [`openapi.json`](../openapi.json), under
`components.schemas.ServerMessage`. The schema for just the game portion is `GameSnapshot`; it references the tagged
`View` and `Action` unions, whose per-game payload schemas are also in that file. This JSON Schema document validates
the server message received on the room WebSocket:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$ref": "../openapi.json#/components/schemas/ServerMessage"
}
```

The snapshot message has this structure (the `$ref`s resolve to the exact schemas in `openapi.json`):

```json
{
  "type": "object",
  "required": ["type", "room", "acknowledged"],
  "properties": {
    "type": { "const": "snapshot" },
    "acknowledged": { "type": "boolean" },
    "room": { "$ref": "../openapi.json#/components/schemas/Snapshot" }
  }
}
```

`room` includes the member and room metadata as well as `game`. The exact game object schema is:

```json
{
  "type": "object",
  "required": ["id", "config", "players", "can_finish", "turn", "legal_actions"],
  "properties": {
    "id": { "type": "string" },
    "config": { "$ref": "../openapi.json#/components/schemas/Config" },
    "players": {
      "type": "array",
      "items": { "$ref": "../openapi.json#/components/schemas/MemberView" }
    },
    "you": { "type": ["integer", "null"], "minimum": 0 },
    "can_finish": { "type": "boolean" },
    "turn": { "$ref": "../openapi.json#/components/schemas/Turn" },
    "outcome": {
      "oneOf": [
        { "type": "null" },
        { "$ref": "../openapi.json#/components/schemas/Outcome" }
      ]
    },
    "view": {
      "oneOf": [
        { "type": "null" },
        { "$ref": "../openapi.json#/components/schemas/View" }
      ]
    },
    "legal_actions": {
      "type": "array",
      "items": { "$ref": "../openapi.json#/components/schemas/Action" }
    }
  }
}
```

For a seated player, `you` and `view` identify that seat and its projection. The nullable forms are part of the shared
protocol schema because the same room snapshot can be delivered to a benched member; in that case `you` and `view` are
null and `legal_actions` is empty. For the full room envelope, enum variants, field formats, and per-game properties,
use the referenced generated schemas rather than duplicating them here.

## Shared game snapshot

Every seated player receives:

- The game ID, public game configuration, and ordered player roster (member IDs and nicknames).
- Their own player index (`you`).
- `can_finish`, indicating whether this viewer may submit the host-only completion control, when supported by the game.
- The eligible player indices and ruleset turn number (`turn`), plus the outcome when the game is complete.
- Their personalized game `view` and their own legal actions for the current state. An ineligible or completed player has
  no legal actions.

The game-specific views below supplement this shared snapshot. Game views do not include the authoritative state, seed,
random-number-generator state, or hidden stock/deck order. Public setup configuration is visible. A player's legal actions
are commands they may submit, not a substitute for the view; actions can contain indices or choices that do not reveal the
value of hidden cards.

## Per-game information

| Game | Every seated player can see |
| --- | --- |
| [Crazy Eights](../crates/arena-core/src/games/crazy_eights/rules.md) | Their own hand; every player's hand size; the top discard and active suit; stock size; play direction; and any pending draw obligation. Other hands, the remaining discard pile, and stock order are hidden. |
| [Mancala — Kalah](../crates/arena-core/src/games/mancala/rules.md) | The complete board, including every house and store stone count. The entire game state is public. |
| [Shithead](../crates/arena-core/src/games/shithead/rules.md) | Setup status; their own hand; for each player, hand count, face-up cards, and face-down card count; the pile's top card and effective top; pile size; stock size; burned-card count; and finishing order. Face-down card values are hidden even from their owner; other hands and stock order are hidden. |
| [Skyjo](../crates/arena-core/src/games/skyjo/rules.md) | Every player's grid with revealed values and removed-slot markers; discard top; stock size; current phase; and finisher, if any. Hidden card values remain hidden even from their owner. A pending drawn card is visible only to the player who drew it; stock order is hidden. |
| [Tic-Tac-Toe](../crates/arena-core/src/games/tic_tac_toe/rules.md) | The complete board, with occupied cells identified by player index. The entire game state is public. |
| [The Crew](../crates/arena-core/src/games/the_crew/rules.md) | Their own hand; every player's hand size; current trick; last completed trick and its winner; tricks won by each player; next card-playing player; and all player communications, including declarations that remain after the declared card is played. Other hands and the seed are hidden. |
| [Tempel des Schreckens](../crates/arena-core/src/games/tempel_des_schreckens/rules.md) | Their own role; their own counts of unrevealed gold, trap, and empty cards (not positions); all rows with face-up card values and hidden positions represented without values; readiness by player; phase; key holder; round and reveals this round; cumulative revealed-pile counts; and the role/card setup distribution. Other roles, other players' private card counts, the unused role, hidden card positions (including their own), and RNG state are hidden. |

Hidden information remains hidden after game completion. In particular, the live view does not turn into a full-state or replay
view when an outcome is set. Completed replay downloads are a separate, intentionally information-revealing feature available
to room members.
