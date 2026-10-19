# Tempel des Schreckens (3–10 players)

## Initial state

- Three to ten players. Shuffle the role pool below and deal one private role to each seat. Omit the unused role without
  revealing it when the pool exceeds the player count. Roles stay fixed throughout the game; each player sees only their
  own role.
- Shuffle all cards and deal five per player. Players know their own counts of gold, traps, and empty cards, but not
  their positions: all cards are placed face-down in shuffled order. Other players' counts are private.
- Seat 0 is the host and chooses any seat, including themselves, to hold the initial key. The key holder selects
  reveals; seat order does not otherwise determine play. Rematches retain seats and require a fresh initial key choice.
- State consists of roles, card rows and visibility, readiness, phase, key holder, round, reveals this round, three
  cumulative revealed piles, deterministic RNG state, action counter, and result.
- Public information: card row sizes and face-up cards, ready seats, key holder, round, round reveal count,
  revealed-pile counts, and the setup distribution. Card positions remain hidden even from their owner. The unused role,
  other roles, and RNG state are private.

| # players | # adventurers | # guardians | # empty | # gold | # traps |
|-----------|---------------|-------------|---------|--------|---------|
| 3         | 2             | 2           | 8       | 5      | 2       |
| 4         | 3             | 2           | 12      | 6      | 2       |
| 5         | 3             | 2           | 16      | 7      | 2       |
| 6         | 4             | 2           | 20      | 8      | 2       |
| 7         | 5             | 3           | 26      | 7      | 2       |
| 8         | 6             | 3           | 30      | 8      | 2       |
| 9         | 6             | 3           | 34      | 9      | 2       |
| 10        | 7             | 4           | 37      | 10     | 3       |

## Actions and eligibility

| Phase      | Legal actions                                                                                 | Result                                                                                                                           |
|------------|-----------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| Choose key | **ChooseKey(player)**, host only, selecting a valid seat.                                     | Set the initial key holder and begin inspection.                                                                                 |
| Inspect    | **Ready**, from any player not ready.                                                         | Confirm that player's card counts have been inspected. Once everyone is ready, begin revealing.                                  |
| Reveal     | **Reveal(player, index)**, key holder only, targeting an unrevealed card of any other player. | Turn the card face-up, add it to its revealed-pile count, and pass the key to its owner. Check the result, then the round limit. |

Readiness is simultaneous; already-ready players wait. No reveals are legal until everyone is ready. There is no
passing, self-reveal, or repeat reveal of a face-up card. Card indices stay fixed within a round.

## Rounds and configuration

Play at most four rounds, with five, four, three, then two cards per player at the start of each round. Each round
permits exactly as many reveals as there are players, unless a winning condition ends play first. This is a total reveal
limit: several cards may be revealed from one player's row, and another row may remain entirely hidden.

After the round's reveal limit, collect and shuffle all unrevealed cards together, then redistribute them equally.
Revealed cards stay out of play; the three revealed-pile counts persist. Reset readiness and begin inspection. The key
stays with the owner of the last revealed card through inspection and into the next round. Roles are not redistributed.
Owners' remaining counts reflect public reveals without exposing card positions.

No configurable rules (`GameConfig` is empty). The setup table fixes the role and card pools. All traps are required for
a trap-based guardian win: two for three to nine players, three for ten players.

## Goal and termination

Adventurers win immediately when the cumulative gold pile contains all gold. Guardians win immediately when the
cumulative trap pile contains all traps, or when the final reveal of round four leaves gold unrevealed. Check gold
completion before the round-four deadline; collecting the last gold on that reveal wins for adventurers.

The winners are exactly the seats assigned the winning role; an unused role is not a player. There are no scores or
draws. Terminal state retains the card rows and key holder without redealing; no actions are legal after the result.
Private roles and unrevealed card positions are not exposed in live views, including after completion; completed replays
contain full hidden information.
