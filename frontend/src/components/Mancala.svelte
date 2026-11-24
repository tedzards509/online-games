<script lang="ts">
  import { t } from '../lib/i18n';
  import type { Action, BoardProps, Schemas } from '../lib/types';
  import Board from './table/Board.svelte';
  import BoardSpace from './table/BoardSpace.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
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
  const yourTurn = $derived(active.includes(you));
  function action(pit: number): Action | undefined {
    return actions.find((a) => a.game === 'mancala' && a.action.pit === pit);
  }
</script>

{#snippet inStore(player: number)}{$t('mancala.inStore', {
    values: { count: view.board[stores[player]] },
  })}{/snippet}

<Board>
  <PlayerList
    players={names.map((_, player) => player).filter((p) => p !== you)}
    {names}
    {you}
    {active}
    details={inStore}
  />
  <TableSurface>
    <!-- Desktop: opponent row on top, stores at the ends. Narrow screens turn
         the board into two columns so pits keep comfortable touch sizes. -->
    <div class="mancala" role="group" aria-label={$t('mancala.board')}>
      <div class="store" aria-label={$t('mancala.opponentStore')}>
        {view.board[stores[1 - you]]}
      </div>
      <div class="pits">
        {#each [...other, ...own] as pit (pit)}
          {@const legal = action(pit)}
          <BoardSpace
            round
            label={$t('mancala.pit', {
              values: { pit: pit + 1, count: view.board[pit] },
            })}
            move={legal && (() => move(legal))}>{view.board[pit]}</BoardSpace
          >
        {/each}
      </div>
      <div class="store" aria-label={$t('mancala.yourStore')}>
        {view.board[stores[you]]}
      </div>
    </div>
  </TableSurface>
  <PlayerDock
    name={names[you]}
    colorIndex={you}
    attention={yourTurn}
    hint={yourTurn ? $t('mancala.hint') : undefined}
  >
    {#snippet details()}{@render inStore(you)}{/snippet}
  </PlayerDock>
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
  /* Two columns between the opponent store (top) and yours (bottom): own pits
     descend on the left and opponent pits ascend on the right, so sowing
     stays counter-clockwise. rtl puts the first-listed (opponent) column on
     the right. */
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
      direction: rtl;
    }
  }
</style>
