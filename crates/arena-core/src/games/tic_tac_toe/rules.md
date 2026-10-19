# Tic-Tac-Toe (2 players)

## Initial state

- Two players; an empty 3×3 board. Seat 0 is X and starts; seat 1 is O.
- Players have a fixed order. The seed has no effect.
- State consists of the board, active seat, and result. All information is public. Cell indices are row-major, 0–8.

## Actions and eligibility

Only the active player may act while the game is unfinished.

- **Place (index):** put your mark in one empty cell. Occupied cells and indices outside 0–8 are illegal.
- After placement, check for a win, then a full-board draw. Otherwise, the opponent takes the next turn.
- There is no pass, moving an existing mark, or removing a mark.

## Configuration

No configurable rules (`GameConfig` is empty): fixed 3×3 board, three in a line, X starts.

## Goal and termination

Form a complete row, column, or diagonal of three of your own marks. The first completed line wins immediately,
including on the last empty cell. A full board without a winning line is a draw, represented by no winners. There are no
scores. No actions are legal after either result.
