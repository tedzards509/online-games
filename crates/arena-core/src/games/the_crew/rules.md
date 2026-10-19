# The Crew (3–5 players, without missions)

## Initial state

- Three to five players; one shuffled 40-card deck: pink, yellow, blue, and green each numbered 1–9, plus black trumps numbered 1–4.
- Deal all cards in seat order, one at a time starting at seat 0. Three players receive 14, 13, and 13 cards; four receive ten each; five receive eight each. Hands are sorted by color then number.
- The holder of black 4 leads the first trick, but need not play black 4. Play proceeds in increasing seat order, wrapping. Rematches retain seats.
- State consists of hands, current and completed tricks, next card-playing seat, each player's communication, action counter, and result. Seat 0 is the host.
- Players see their own hand, all hand sizes, current and last completed trick (including its winner), trick counts, next player, and all communications. Other hands and the seed remain private.

## Actions and eligibility

| Action | Eligibility | Result |
| --- | --- | --- |
| **Play(index)** | Only the next card-playing seat; valid index into their current hand. | Play one card face-up. Follow the led color, including black, whenever possible; otherwise any card is legal. The first card sets the led color. |
| **Communicate(index, declaration)** | Any player who has not communicated, before the first trick or between complete tricks, until the leader plays. | Publicly identify a colored card still in hand as lowest, only, or highest of that color. The card stays in hand; the once-per-game communication is spent. |

No passing or drawing. A trick contains exactly one card per player. The highest black trump wins; without a trump, the highest card of the led color wins. Off-color nontrumps cannot win. The winner leads the next trick. No cards are recycled.

Separately from playable actions, the host may use the room's **Finish(won)** command at any time while unfinished, including during a trick. It records an internal finish action for replay but is not offered as a card-game move or as turn eligibility in room snapshots.

Communication does not change the next card-playing seat. Lowest/highest declarations compare against the player's currently held cards of that color; only requires exactly one such card. A lone colored card may be declared lowest, only, or highest. Black cards and nonextreme colored cards cannot be communicated. The declaration is a statement at communication time, not a restriction on future plays; it remains public after that card is played. Communication is optional and cannot occur during a trick.

Concurrent actions are revalidated in queue order: communication queued after a lead becomes illegal, while communication queued before it does not prevent the lead.

## Configuration

No configurable rules (`GameConfig` is empty). Missions are not implemented; there are no mission objectives, automatic success checks, or individual trick-based winners.

## Goal and termination

The group decides whether it has won or lost; only the host records that decision using **Finish**. A win lists every seat as a winner; a loss lists no winners. Neither result assigns scores.

When a completed trick leaves anyone without cards, no further trick or communication is available; wait for the host's decision rather than ending automatically. With three players this leaves one unplayed card in seat 0's hand after thirteen tricks. With four or five players all cards are played after ten or eight tricks respectively. The host may also finish earlier, leaving cards and a partial trick in place. No actions are legal after either result.
