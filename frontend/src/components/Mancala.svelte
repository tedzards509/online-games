<script lang="ts">
  import type { Action, BoardProps, Schemas } from '../lib/types';
  import Board from './table/Board.svelte';
  import BoardSpace from './table/BoardSpace.svelte';
  import PlayerList from './table/PlayerList.svelte';
  import TableSurface from './table/TableSurface.svelte';

  let {
    view,
    actions,
    names,
    you,
    active,
    move,
  }: BoardProps<Schemas['MancalaView']> = $props();
  /** Board indices of each player's store. */
  const stores = [6, 13];
  const own = $derived(you === 0 ? [0, 1, 2, 3, 4, 5] : [7, 8, 9, 10, 11, 12]);
  const other = $derived(
    you === 0 ? [12, 11, 10, 9, 8, 7] : [5, 4, 3, 2, 1, 0],
  );
  function action(pit: number): Action | undefined {
    return actions.find((a) => a.game === 'mancala' && a.action.pit === pit);
  }
</script>

<Board>
  <PlayerList players={names.map((_, player) => player)} {names} {you} {active}>
    {#snippet details(player)}{view.board[stores[player]]} in store{/snippet}
  </PlayerList>
  <TableSurface>
    <!-- Desktop: opponent row on top, stores at the ends. Narrow screens turn
         the board into two columns so pits keep comfortable touch sizes. -->
    <div class="mancala" role="group" aria-label="Mancala board">
      <div class="store" aria-label="Opponent store">
        {view.board[stores[1 - you]]}
      </div>
      <div class="pits">
        {#each [...other, ...own] as pit (pit)}
          {@const legal = action(pit)}
          <BoardSpace
            round
            label={`Pit ${pit + 1}: ${view.board[pit]} stones`}
            move={legal && (() => move(legal))}>{view.board[pit]}</BoardSpace
          >
        {/each}
      </div>
      <div class="store" aria-label="Your store">
        {view.board[stores[you]]}
      </div>
    </div>
  </TableSurface>
</Board>

<style>
  .mancala {
    display: grid;
    grid-template-columns: 4rem minmax(0, 1fr) 4rem;
    gap: 0.75rem;
    width: min(100%, 44rem);
    font-size: 1.4rem;
  }
  .pits {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 0.75rem;
  }
  .store {
    display: grid;
    place-items: center;
    border: var(--arena-line);
    border-radius: 2rem;
    background: var(--arena-surface);
    color: var(--arena-text);
    font-weight: 800;
  }
  /* Two columns: opponent pits descend on the left, own pits on the right,
     keeping the counter-clockwise sowing loop between the stores. */
  @media (max-width: 40rem) {
    .mancala {
      grid-template-columns: minmax(0, 14rem);
      grid-template-rows: 3.5rem auto 3.5rem;
      justify-content: center;
    }
    .pits {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      grid-template-rows: repeat(6, auto);
      grid-auto-flow: column;
    }
  }
</style>
