<script lang="ts">
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    you,
    move,
  }: {
    view: Schemas['TicTacToeView'];
    actions: Action[];
    you: number;
    move: (action: Action) => void;
  } = $props();

  function action(index: number): Action | undefined {
    return actions.find(
      (a) => a.game === 'tic_tac_toe' && a.action.index === index,
    );
  }
</script>

<p class="muted">You: {you === 0 ? 'X' : 'O'}</p>
<div class="board" aria-label="Tic-Tac-Toe board">
  {#each view.board as cell, index}
    {@const legal = action(index)}
    {@const mark = cell === null ? '' : cell === 0 ? 'X' : 'O'}
    <button
      disabled={!legal}
      aria-label={`Cell ${index + 1}: ${mark || 'empty'}`}
      onclick={() => legal && move(legal)}>{mark}</button
    >
  {/each}
</div>

<style>
  .board {
    display: grid;
    grid-template-columns: repeat(3, 72px);
    gap: 6px;
    width: max-content;
    margin: 24px auto;
  }
  .board button {
    height: 72px;
    padding: 0;
    font-size: 2rem;
  }
  .board button:disabled {
    opacity: 1;
  }
</style>
