# Shithead (2–5 players)

## Initial state

- Two to five players; one shuffled 52-card deck. Each player receives three face-down cards and seven hand cards, with
  no face-up cards yet. The remaining cards form the stock; the play pile and burned-card count start empty.
- Setup is simultaneous: all players who have not chosen their face-up cards may act. After everyone has chosen, seat 0
  starts ordinary play; play proceeds in increasing seat order, wrapping and skipping finished players. Seats follow
  a fixed player order throughout the game.
- State consists of configuration, setup status, each player's three card zones, stock, play pile, burned-card count,
  finishing order, eligible seats, and result.
- Each player sees their own hand, all face-up cards, hand/face-down counts, pile top/size, effective top beneath
  transparent threes, stock size, burned count, setup status, and finishing order. Face-down values are hidden even from
  their owner; other hands and stock order remain private.

## Actions and eligibility

During setup, **SelectFaceUp (indices)** selects exactly three distinct hand cards in ascending index order and places
them face-up over the three face-down cards. Four cards remain in hand. The player then waits; no ordinary actions are
legal until everyone has selected. No card ranks restrict this selection.

During ordinary play, only the active, unfinished player may act. Use the first nonempty zone: hand, then face-up, then
face-down. Picking up cards makes the hand the active zone again.

- **Play (indices):** from hand or face-up cards, play any nonempty subset of cards of one rank. Indices refer to that
  zone and must be valid, unique, and ascending. The rank must be legal against the effective top before play, except
  for enabled four-card bombs; suits do not matter.
- **Blind (index):** only when hand and face-up cards are empty, choose one face-down card without seeing it. If legal,
  play it normally. Otherwise, add it to the pile and take the entire pile into your hand; the turn ends.
- **Pickup:** whenever the pile is nonempty, take the whole pile into your hand and end your turn, even if you could
  play. With no legal play and no blind choice, pickup is mandatory. There is no pass, stock-draw action, or pickup from
  an empty pile.

After a play or blind attempt, draw from stock until the hand has three cards or stock is exhausted. Never recycle the
pile or burned cards into stock. After refill, a player with no cards in any zone finishes and is recorded once in
finishing order.

## Card effects and configuration

Ignore any consecutive threes at the top when determining the effective top; a pile consisting only of threes has no
rank restriction. Threes still count as physical cards for pickup and consecutive-four burns.

| Rank / condition                              | Effect                                                                                                                                         |
|-----------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| Ordinary rank                                 | Must equal or exceed the effective top (ace high), except that on a 7 it must be at most 7. Any rank is legal with no effective top or on a 2. |
| 2                                             | Legal on any pile; resets the rank restriction for the next play without clearing the pile.                                                    |
| 3                                             | Legal at any time during ordinary play; transparent for rank comparison.                                                                       |
| 7                                             | Legal on an effective top of at most 7 (or no restriction). Subsequent cards must be at most 7, including a 10, which is therefore not legal.  |
| 10                                            | Legal except on an effective 7; burns the whole pile.                                                                                          |
| Four consecutive cards of one rank at the top | Burns the whole pile, including when formed across multiple turns or in one multi-card play.                                                   |

Burning removes all pile cards permanently and gives the same player another turn unless they have finished. Otherwise
play passes to the next unfinished seat. No other ranks have special effects.

Configuration fixes `four_card_bombs` at setup (default false). When true, four matching cards played together
bypass all rank restrictions, including a 7, and burn the pile. When false, four matching cards must be legal normally
to burn. This does not permit mixed ranks or an otherwise illegal group of fewer than four cards. All other special
rules are non-configurable.

## Goal and termination

Get rid of all cards in all three zones, after any required refill. Finished players take no further turns. When only
one unfinished player remains, the game ends: that player loses and every finished player wins. Winners are listed in
finishing order; there are no placement points or scores. No actions are legal after the result.
