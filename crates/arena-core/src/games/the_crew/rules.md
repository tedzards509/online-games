# The Crew (3–5 players, without missions)

## Initial state

- Three to five players; one shuffled 40-card deck: pink, yellow, blue, and green each numbered 1–9, plus black trumps
  numbered 1–4.
- Deal all cards in seat order, one at a time starting at seat 0. Three players receive 14, 13, and 13 cards; four
  receive ten each; five receive eight each. Hands are sorted by color then number.
- The holder of black 4 leads the first trick, but need not play black 4. Play proceeds in increasing seat order,
   wrapping.
- State consists of hands, current and completed tricks, next card-playing seat, each player's communication, action
   counter, and result.
- Players see their own hand, all hand sizes, current and last completed trick (including its winner), trick counts,
  next player, original black-four holder, and all communications including whether each declared card has been played.
  Other hands and the seed remain private.

## Actions and eligibility

| Action                              | Eligibility                                                                                                     | Result                                                                                                                                                      |
|-------------------------------------|-----------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Play(index)**                     | Only the next card-playing seat; valid index into their current hand.                                           | Play one card face-up. Follow the led color, including black, whenever possible; otherwise any card is legal. The first card sets the led color.            |
| **Communicate(index, declaration)** | Any player who has not communicated, before the first trick or between complete tricks, until the leader plays. | Publicly identify a colored card still in hand as lowest, only, or highest of that color. The card stays in hand; the once-per-game communication is spent. |

No passing or drawing. A trick contains exactly one card per player. The highest black trump wins; without a trump, the
highest card of the led color wins. Off-color nontrumps cannot win. The winner leads the next trick. No cards are
recycled.

An external **SharedResult(won)** decision may finish the game at any time while unfinished, including during a trick.
It is independent of player actions and turn eligibility.

Communication does not change the next card-playing seat. Lowest/highest declarations compare against the player's
currently held cards of that color; only requires exactly one such card. A lone colored card may be declared lowest,
only, or highest. Black cards and nonextreme colored cards cannot be communicated. The declaration is a statement at
communication time, not a restriction on future plays; it remains public after that card is played. Communication is
optional and cannot occur during a trick.

Communication is legal before a lead, not after it. Communicating does not prevent the leader from playing.

## Configuration

No configurable rules (`GameConfig` is empty). Missions are not implemented; there are no mission objectives, automatic
success checks, or individual trick-based winners.

## Goal and termination

The group supplies its shared win or loss as an external result. A win lists every
seat as a winner; a loss lists no winners. Neither result assigns scores.

When a completed trick leaves anyone without cards, no further trick or communication is available; await the external
result rather than ending automatically. With three players this leaves one unplayed card in seat 0's hand after
thirteen tricks. With four or five players all cards are played after ten or eight tricks respectively. An earlier
result leaves remaining cards and a partial trick in place. No actions or resolutions are legal after either result.
