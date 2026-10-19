# Crazy Eights (2–6 players)

## Initial state

- One shuffled 52-card deck; seven cards per player with two players, five with three to six players.
- One face-up discard; the remaining cards form the stock. The opening discard has no special effect; its printed suit
  is the active suit.
- Players have a fixed order; seat 0 starts, initially towards increasing seat indices. No draw
  obligation is pending. Seating and configuration are fixed for each game.
- State consists of hands, stock, discard pile, active suit, direction, active seat, pending draw obligation, and
  result. Each player sees only their own hand, other hand sizes, stock size, and public state.

## Actions and eligibility

Only the active player may act, and only while the game is unfinished. Each action ends that player's turn.

- **Play (index, suit):** play one card from hand. Without a pending obligation, it must match the active suit or the
  top discard's rank, or be an eight. An eight requires a chosen suit; every other card forbids a suit choice and sets
  the active suit to its printed suit.
- **Draw:** without a pending obligation, draw one card, even if a card could be played. The drawn card cannot be played
  in the same turn.
- **Pending ace:** either play any ace, regardless of suit, or draw two. A responding ace clears the obligation; it does
  not start another one. No other card, including an eight, may be played.
- **Pending two:** draw two; no card may be played and penalties do not stack.

Recycle all discards beneath the top when stock runs out. Draw as many cards as available if the requested count cannot
be met; clear any draw obligation even then. A draw taking no cards is a pass and ends only that turn.

## Card effects and configuration

Configuration fixes four independent booleans at setup; all default to false. Disabled special ranks behave as
ordinary matching cards. Eights are always wild, not configurable.

| Rank | Effect after play                                                                        | Configuration        |
|------|------------------------------------------------------------------------------------------|----------------------|
| Ace  | Unless responding to a pending ace, require the next player to play an ace or draw two.  | `ace_are_you_strong` |
| J    | Skip the next player's turn in the current direction.                                    | `jack_skip`          |
| 10   | Reverse direction; with two players, skip the opponent instead.                          | `ten_reverse`        |
| 8    | Play regardless of rank/suit when no obligation is pending; choose the next active suit. | Always enabled       |
| 2    | Require the next player to draw two.                                                     | `two_draw_two`       |

Otherwise, play proceeds to the next seat in the current direction.

## Goal and termination

The first player to empty their hand wins immediately, including when playing a special card; no opponent must resolve a
final penalty. Passing never ends the game in a draw, even if all players repeatedly pass. No actions are legal after a
win.
