# Initial rules

These are explicit defaults, not a claim that every house variant is covered. Seats follow join order. The host begins except for Skyjo's post-setup starting-player selection. Games use server-shuffled decks; rematches keep the same seats.

## Mancala — Kalah (2 players)

Six pits per side, four stones per pit. Sow counterclockwise, including your store and skipping the opponent's store. Ending in your store gives another turn. Ending in an empty pit on your side captures that stone and the nonempty opposite pit. If either side empties, sweep the other side into its store. Most stones wins; ties are draws (both seats are winners).

Internally pits 0–5 belong to seat 0, store 6; pits 7–12 belong to seat 1, store 13. Opposite pit is `12 - pit`. The UI always puts your side at the bottom.

## Crazy Eights (2–6 players)

One 52-card deck; seven cards each for two players, five otherwise. Match the discard's rank or active suit. The host chooses five independent boolean options before creating the room; disabled ranks behave as ordinary matching cards. Defaults enable only wild eights, preserving the original rules.

- **Ace — Are you strong:** the next player may play any ace (regardless of suit) or draw two and end their turn. Another ace passes on the same two-card obligation; penalties do not accumulate.
- **J — jump:** skip the next player.
- **10 — reverse:** reverse direction; with two players skip the opponent instead.
- **8 — choose suit:** an eight is always playable and chooses the next suit, except during a draw-two obligation.
- **2 — draw two:** the next player must draw two and end their turn; no stacking or card response.

The opening discard never triggers special effects; its printed suit applies. Playing the last card wins immediately, including a special card. Draw penalties recycle as needed and draw as many cards as remain if fewer than two are available.

Play one card per turn. You must play if you can; otherwise draw one and end your turn, even if the drawn card could be played. Recycle the discard beneath its top when stock empties. If no stock/discard can be recycled, pass; a full cycle of blocked players draws the game. First empty hand wins.

## Shithead (2–5 players)

One 52-card deck. Each player receives three face-down cards, three face-up cards, and three hand cards. No initial swapping. Play from hand first, then face-up, then face-down; choose face-down cards blindly. Refill your hand to three after playing while stock remains. Multiple cards of the same rank may be played together; indices must be unique and ascending.

Play equal or higher than the top rank (ace high). **2** may be played on anything and resets the pile. **10** may be played on anything and burns the pile. Four equal consecutive ranks also burn it. Burning gives another turn unless you have gone out. No other special ranks. Picking up the whole pile voluntarily ends your turn. A blind card that cannot be played joins the pile and you pick it all up, ending your turn.

Finished players are skipped. Continue until one player remains: that player loses; everyone else wins. Finishing order is recorded. There are no placement points.

## Skyjo (2–8 players, one round)

150 cards: five −2, fifteen 0, ten each of −1 and 1–12. Each player has a face-down 3×4 grid. In seat order, reveal two cards of your choice. The highest revealed sum starts; ties go to the earlier seat.

On your turn either take the visible discard and replace any remaining grid card, or draw privately from stock. A stock draw can replace any remaining card, or be discarded followed by revealing one hidden grid card. All replacements become face-up. Three equal, revealed cards in a vertical column remove that column to the discard. Recycle the discard beneath its top when needed.

The first player with no hidden cards triggers the last turn for every other player. Then reveal all cards, remove matching columns, and total remaining values. Double the finisher's positive score if it is not **strictly** lower than every opponent's. Lowest score wins; tied lowest players share the win. No cumulative 100-point match yet.

Grid indices are row-major: 0–3, 4–7, 8–11. Removed slots stay in place and cannot be targeted. A draw from an empty, unrecyclable stock is unavailable; the discard remains usable.

## Tic-Tac-Toe (2 players)

Seat 0 is X and starts; seat 1 is O. Alternate placing one mark in an empty cell of a 3×3 board. Three of your marks in a row, column, or diagonal wins immediately. A full board without a winning line is a draw. No moves are accepted after either result. Indices are row-major, 0–8; all board information is public. The seed has no effect.
