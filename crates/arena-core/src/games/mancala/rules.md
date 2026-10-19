# Mancala — Kalah (2 players)

## Initial state

- Two players, each with six houses containing four stones each and an empty store: 48 stones in total.
- Seat 0 starts; seats follow the host-selected player order and remain fixed for the game. The seed has no effect.
- State consists of the stone counts, active seat, and result. All information is public.
- Houses 0–5 and store 6 belong to seat 0; houses 7–12 and store 13 belong to seat 1. Opposite houses have indices
  summing to 12.

## Actions and eligibility

Only the active player may act while the game is unfinished. There is no pass.

- **Sow (pit):** choose a nonempty house on your own side. Stores, empty houses, and opponent houses are not legal
  choices.
- Empty that house and distribute its stones one at a time counterclockwise (increasing indices, wrapping after 13),
  including your store but skipping the opponent's store. Continue around the board as many times as necessary.
- If the last stone lands in your own house that was empty immediately before that stone landed, and its opposite house
  is nonempty, move the landing stone and all opposite stones into your store.
- After resolving capture, check for termination. Otherwise, ending in your own store gives another turn; any other
  landing passes play to the opponent.

## Configuration

No configurable rules (`GameConfig` is empty). This is Kalah with six houses per side, four initial stones per house,
captures requiring a nonempty opposite house, and no pie/swap rule.

## Goal and termination

Collect the most stones in your store. After each sow and capture, if either player's houses are all empty, sweep the
stones remaining in each player's houses into that player's store and end immediately, even if the last stone landed in
a store.

Scores are final store counts in seat order. The higher score wins; equal scores are a draw represented by no winners.
No actions are legal after the result. Stones are never created or removed.
