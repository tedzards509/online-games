# Mancala — Kalah (2 players)

Seats follow join order; the host begins. Rematches keep the same seats.

Six pits per side, four stones per pit. Sow counterclockwise, including your store and skipping the opponent's store. Ending in your store gives another turn. Ending in an empty pit on your side captures that stone and the nonempty opposite pit. If either side empties, sweep the other side into its store. Most stones wins; ties are draws (both seats are winners).

Internally pits 0–5 belong to seat 0, store 6; pits 7–12 belong to seat 1, store 13. Opposite pit is `12 - pit`. The UI always puts your side at the bottom.
