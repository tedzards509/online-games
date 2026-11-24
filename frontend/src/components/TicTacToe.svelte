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
  }: BoardProps<Schemas['TicTacToeView']> = $props();
  const marks = ['X', 'O'];
  const yourTurn = $derived(active.includes(you));

  function action(index: number): Action | undefined {
    return actions.find(
      (a) => a.game === 'tic_tac_toe' && a.action.index === index,
    );
  }
</script>

{#snippet mark(player: number)}
  <span class="mark" class:o={player === 1}>{marks[player]}</span>
{/snippet}

<Board>
  <PlayerList
    players={names.map((_, player) => player).filter((p) => p !== you)}
    {names}
    {you}
    {active}
    details={mark}
  />
  <TableSurface>
    <div class="cells" role="group" aria-label={$t('ticTacToe.board')}>
      {#each view.board as cell, index}
        {@const legal = action(index)}
        {@const mark = cell === null ? '' : marks[cell]}
        <BoardSpace
          label={$t('ticTacToe.cell', {
            values: { index: index + 1, mark: mark || $t('ticTacToe.empty') },
          })}
          move={legal && (() => move(legal))}
        >
          <span class:o={cell === 1}>{mark}</span>
        </BoardSpace>
      {/each}
    </div>
  </TableSurface>
  <PlayerDock
    name={names[you]}
    colorIndex={you}
    attention={yourTurn}
    hint={yourTurn ? $t('ticTacToe.hint') : undefined}
  >
    {#snippet details()}{@render mark(you)}{/snippet}
  </PlayerDock>
</Board>

<style>
  .mark {
    font-size: 1.2rem;
    font-weight: 800;
  }
  .cells {
    display: grid;
    grid-template-columns: repeat(3, 5rem);
    gap: 0.75rem;
    font-size: 2.5rem;
    color: var(--arena-card-dark);
  }
  .o {
    color: var(--arena-card-red);
  }
</style>
