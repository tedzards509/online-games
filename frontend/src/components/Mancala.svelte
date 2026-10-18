<script lang="ts">
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    you,
    move,
  }: {
    view: Schemas['MancalaView'];
    actions: Action[];
    you: number;
    move: (action: Action) => void;
  } = $props();
  const own = $derived(you === 0 ? [0, 1, 2, 3, 4, 5] : [7, 8, 9, 10, 11, 12]);
  const other = $derived(
    you === 0 ? [12, 11, 10, 9, 8, 7] : [5, 4, 3, 2, 1, 0],
  );
  function action(pit: number): Action | undefined {
    return actions.find((a) => a.game === 'mancala' && a.action.pit === pit);
  }
</script>

<div class="mancala" aria-label="Mancala board">
  <div class="store" aria-label="Opponent store">
    {view.board[you === 0 ? 13 : 6]}
  </div>
  <div class="pits">
    {#each [other, own] as row}
      <div class="pit-row">
        {#each row as pit}
          {@const legal = action(pit)}
          <button
            class="pit"
            disabled={!legal}
            aria-label={`Pit ${pit + 1}: ${view.board[pit]} stones`}
            onclick={() => legal && move(legal)}>{view.board[pit]}</button
          >
        {/each}
      </div>
    {/each}
  </div>
  <div class="store" aria-label="Your store">
    {view.board[you === 0 ? 6 : 13]}
  </div>
</div>
