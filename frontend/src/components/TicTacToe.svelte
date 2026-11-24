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
  }: BoardProps<Schemas['TicTacToeView']> = $props();
  const marks = ['X', 'O'];

  function action(index: number): Action | undefined {
    return actions.find(
      (a) => a.game === 'tic_tac_toe' && a.action.index === index,
    );
  }
</script>

<Board>
  <PlayerList players={names.map((_, player) => player)} {names} {you} {active}>
    {#snippet details(player)}
      <span class="mark" class:o={player === 1}>{marks[player]}</span>
    {/snippet}
  </PlayerList>
  <TableSurface>
    <div class="cells" role="group" aria-label="Tic-Tac-Toe board">
      {#each view.board as cell, index}
        {@const legal = action(index)}
        {@const mark = cell === null ? '' : marks[cell]}
        <BoardSpace
          label={`Cell ${index + 1}: ${mark || 'empty'}`}
          move={legal && (() => move(legal))}
        >
          <span class:o={cell === 1}>{mark}</span>
        </BoardSpace>
      {/each}
    </div>
  </TableSurface>
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
