<script lang="ts">
  import type { Snippet } from 'svelte';
  import PlayerBar from './PlayerBar.svelte';

  /** A labelled list of player bars, with optional public details per player. */
  let {
    label = 'Players',
    players,
    names,
    you,
    active,
    details,
  }: {
    label?: string;
    /** Player indices to show, in display order. */
    players: number[];
    names: string[];
    you: number;
    active: number[];
    details?: Snippet<[number]>;
  } = $props();
</script>

<ul aria-label={label}>
  {#each players as player (player)}
    <li>
      <PlayerBar
        name={names[player]}
        colorIndex={player}
        you={player === you}
        active={active.includes(player)}
      >
        {#if details}{@render details(player)}{/if}
      </PlayerBar>
    </li>
  {/each}
</ul>

<style>
  ul {
    display: flex;
    flex-wrap: wrap;
    gap: var(--arena-space-1) var(--arena-space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
