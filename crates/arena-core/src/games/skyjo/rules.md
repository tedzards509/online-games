# Skyjo (2–8 players, one round)

## Initial state

- Two to eight players; one shuffled 150-card deck: five −2, fifteen 0, and ten each of −1 and 1–12.
- Deal each player twelve face-down cards in a 3×4 grid; turn one further card face-up as the discard. Remaining cards form the stock.
- Start in setup with seat 0. Seats follow join order, wrap in increasing order, and remain unchanged on rematches.
- State consists of grids (values, visibility, removed slots), stock, discard pile, active seat, phase, pending drawn card/source, first finisher, and result.
- Grid indices are row-major: 0–3, 4–7, 8–11. All revealed values and removed slots are public; hidden values are unknown even to their owner. Discard top, stock size, phase, and finisher are public. A pending stock draw is visible only to its active player; stock order remains private.

## Actions and eligibility

Only the active player may act while the round is unfinished. A turn can require multiple actions; the active seat remains unchanged until the turn is complete. Removed slots cannot be targeted.

| Phase | Legal actions | Result |
| --- | --- | --- |
| Setup | **Reveal(index)** on one hidden grid card. | Reveal exactly two cards per seat, then move to the next seat. After all seats, the highest revealed sum starts ordinary play; ties go to the earlier seat. |
| Draw | **Draw(discard)** or **Draw(stock)**. | Take one card and enter Place. Stock is available only if nonempty or recyclable. |
| Place | **Replace(index)** on any occupied slot, hidden or revealed. | Put the drawn card face-up there; put the old card on the discard; complete the turn. |
| Place, stock source only | **Discard**, only if at least one hidden grid card remains. | Put the drawn card on the discard and enter Reveal; do not end the turn yet. A card taken from discard must be placed. |
| Reveal | **Reveal(index)** on one hidden grid card. | Turn it face-up and complete the turn. |

There is no pass or direct grid reveal during Draw. When stock is empty, shuffle all discards beneath the top into stock, keeping the top discard in place. A draw from empty, unrecyclable stock is unavailable; taking the discard remains legal.

## Column removal and configuration

At the end of each ordinary turn, remove every vertical column containing three equal, revealed cards in the acting player's grid. Append removed values to the discard; removed slots remain in place and score nothing. Rows and diagonals do not qualify. Setup does not remove columns.

No configurable rules (`GameConfig` is empty). This implementation plays one round, not a cumulative match to 100 points.

## Goal and termination

Minimize the sum of remaining card values. After column removal, the first player with no hidden cards is the finisher (an entirely removed grid also qualifies). Every other player then receives exactly one complete final turn in seat order; the finisher receives none, and a later player revealing everything does not restart the final cycle.

When play would return to the finisher, reveal all remaining cards and remove all newly matching columns from every grid before scoring. Sum each player's remaining values. Double the finisher's score only if it is positive and at least one opponent's score is less than or equal to it. Zero and negative scores are never doubled.

Scores are listed in seat order; all players tied for the lowest final score win. No actions are legal after the result.
