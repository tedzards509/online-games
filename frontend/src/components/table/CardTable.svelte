<script lang="ts">
  import type { Snippet } from 'svelte';
  import TableSurface from './TableSurface.svelte';

  /** Viewport-filling card game layout: opponents seated around a green
   * center, the viewer's dock at the bottom. Opponents are given in seat
   * order after the viewer (left, across, right). Render it as the board's
   * root for a game listed in `cardTableGames`. */
  let {
    opponents,
    opponent,
    center,
    dock,
  }: {
    /** Opponent player indices in seat order after the viewer. */
    opponents: number[];
    /** One opponent's seat, usually an `OpponentSeat`. */
    opponent: Snippet<[number]>;
    /** Content of the green playing area. */
    center: Snippet;
    /** The viewer's `PlayerDock`. */
    dock: Snippet;
  } = $props();
</script>

<div class="card-table">
  <div
    class="arena"
    data-count={opponents.length}
    style:--opponents={opponents.length}
  >
    <div class="opponents">
      {#each opponents as player (player)}<div class="opponent">
          {@render opponent(player)}
        </div>{/each}
    </div>
    <div class="center"><TableSurface>{@render center()}</TableSurface></div>
  </div>
  <!-- The table fits the viewport, so the whole dock stays one grid row. -->
  <div class="seat">{@render dock()}</div>
</div>

<style>
  .card-table {
    /* One legible 72 × 101px card at default text size, in every context.
       Tight layouts scroll within the viewport instead of shrinking cards. */
    --arena-card-width: 4.5rem;
    min-width: 0;
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: minmax(0, 1fr) auto;
    gap: 0.5rem;
    /* Seats across sit in the middle columns, clear of the corner buttons. */
    padding: 1rem max(1rem, env(safe-area-inset-right)) 1rem
      max(1rem, env(safe-area-inset-left));
    color: var(--arena-text);
    background: var(--arena-canvas);
  }
  .arena {
    min-width: 0;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    /* The middle row is at least as tall as the side seats and the center. */
    grid-template-rows: auto 1fr;
    align-items: center;
    gap: 0.5rem;
    overflow: auto;
  }
  .opponents {
    display: contents;
  }
  .opponent {
    min-width: calc(2 * var(--arena-card-width) + 0.5rem);
  }
  /* Seats: one left, the middle ones across, one right. */
  .opponent:first-child {
    grid-area: 2 / 1;
  }
  .arena[data-count='1'] .opponent:first-child {
    grid-area: 1 / 1 / 2 / 5;
  }
  .arena[data-count='2'] .opponent:first-child {
    grid-area: 1 / 1 / 2 / 3;
  }
  .arena[data-count='2'] .opponent:nth-child(2) {
    grid-area: 1 / 3 / 2 / 5;
  }
  .arena[data-count='3'] .opponent:nth-child(2) {
    grid-area: 1 / 2 / 2 / 4;
  }
  .arena[data-count='3'] .opponent:nth-child(3) {
    grid-area: 2 / 4;
  }
  .arena[data-count='4'] .opponent:nth-child(2) {
    grid-area: 1 / 2;
  }
  .arena[data-count='4'] .opponent:nth-child(3) {
    grid-area: 1 / 3;
  }
  .arena[data-count='4'] .opponent:nth-child(4) {
    grid-area: 2 / 4;
  }
  .center {
    grid-area: 2 / 2 / 3 / 4;
    min-width: 0;
    min-height: calc(var(--arena-card-width) * 1.4 + 6rem);
    display: grid;
  }
  .arena[data-count='1'] .center,
  .arena[data-count='2'] .center {
    grid-column: 1 / -1;
  }
  .center > :global(*) {
    min-width: 0;
    min-height: 0;
    padding: var(--arena-space-2);
  }
  .seat {
    display: grid;
    gap: var(--arena-space-1);
    min-width: 0;
  }
  @media (max-width: 40rem) {
    .card-table {
      padding: 0.5rem 0.5rem calc(4.5rem + env(safe-area-inset-bottom));
      gap: 0.5rem;
    }
    .arena {
      gap: 0.25rem;
    }
  }
  @media (max-height: 500px) and (min-width: 40.01rem) {
    .card-table {
      padding: 0.5rem 5rem;
      gap: 0.25rem;
    }
    .arena {
      grid-template-columns: repeat(var(--opponents), minmax(0, 1fr));
    }
    .arena[data-count] .opponent:nth-child(n) {
      grid-column: auto;
      grid-row: 1;
    }
    .arena[data-count] .center {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
  /* Portrait phones: compact opponents in a scrolling row above the center.
     The column is reversed so that, when the arena must scroll, it starts
     scrolled to the center rather than to the opponents. */
  @media (max-width: 40rem) and (orientation: portrait) {
    .arena {
      display: flex;
      flex-direction: column-reverse;
      align-items: stretch;
      gap: 0.5rem;
    }
    .opponents {
      display: flex;
      flex-wrap: nowrap;
      justify-content: start;
      min-width: 0;
      gap: 0.25rem;
      margin-top: auto;
      overflow-x: auto;
      flex-shrink: 0;
    }
    .opponent {
      /* Four seats fit a 390px screen; more scroll sideways. */
      --arena-card-width: 2.5rem;
      flex: 1 0 5.5rem;
    }
    .center {
      order: -1;
      flex-shrink: 0;
      min-height: 0;
      /* With the opponents' top margin, spare height centers the column. */
      margin-bottom: auto;
    }
    .center > :global(*) {
      padding: var(--arena-space-1);
    }
  }
  @media (max-width: 22.5rem) {
    .arena {
      grid-template-columns: repeat(var(--opponents), minmax(0, 1fr));
    }
    .arena[data-count] .opponent:nth-child(n) {
      grid-column: auto;
      grid-row: 1;
    }
    .arena[data-count] .center {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
</style>
