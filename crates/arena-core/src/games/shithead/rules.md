# Shithead (2–5 players)

## Initial state

- Two to five players; one shuffled 52-card deck. Each player receives three face-down cards, three face-up cards, and three hand cards. The remaining cards form the stock; the play pile and burned-card count start empty.
- No initial swapping. Seat 0 starts; play proceeds in increasing seat order, wrapping and skipping finished players. Rematches keep the same seats.
- State consists of each player's three card zones, stock, play pile, burned-card count, finishing order, active seat, and result.
- Each player sees their own hand, all face-up cards, hand/face-down counts, pile top/size, stock size, burned count, and finishing order. Face-down values are hidden even from their owner; other hands and stock order remain private.

## Actions and eligibility

Only the active, unfinished player may act while the game is unfinished. Use the first nonempty zone: hand, then face-up, then face-down. Picking up cards makes the hand the active zone again.

- **Play(indices):** from hand or face-up cards, play any nonempty subset of cards of one rank. Indices refer to that zone and must be valid, unique, and ascending. The rank must be legal against the pile before play; suits do not matter.
- **Blind(index):** only when hand and face-up cards are empty, choose one face-down card without seeing it. If legal, play it normally. Otherwise, add it to the pile and take the entire pile into your hand; the turn ends.
- **Pickup:** whenever the pile is nonempty, take the whole pile into your hand and end your turn, even if you could play. With no legal play and no blind choice, pickup is mandatory. There is no pass, stock-draw action, or pickup from an empty pile.

After a play or blind attempt, draw from stock until the hand has three cards or stock is exhausted. Never recycle the pile or burned cards into stock. After refill, a player with no cards in any zone finishes and is recorded once in finishing order.

## Card effects and configuration

No configurable rules (`GameConfig` is empty).

| Rank / condition | Effect |
| --- | --- |
| Ordinary rank | Must equal or exceed the top rank (ace high); any rank is legal on an empty pile or a 2. |
| 2 | Legal on any pile; resets the rank restriction for the next play without clearing the pile. |
| 10 | Legal on any pile; burns the whole pile. |
| Four consecutive cards of one rank at the top | Burns the whole pile, including when formed across multiple turns or in one multi-card play. |

Burning removes all pile cards permanently and gives the same player another turn unless they have finished. Otherwise play passes to the next unfinished seat. No other ranks have special effects.

## Goal and termination

Get rid of all cards in all three zones, after any required refill. Finished players take no further turns. When only one unfinished player remains, the game ends: that player loses and every finished player wins. Winners are listed in finishing order; there are no placement points or scores. No actions are legal after the result.
