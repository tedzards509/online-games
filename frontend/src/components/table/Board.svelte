<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Root of every game board: its sections stacked with the shared rhythm. */
  let { children }: { children: Snippet } = $props();
</script>

<div class="board">{@render children()}</div>

<style>
  .board {
    display: grid;
    gap: var(--arena-space-3);
    min-width: 0;
  }
</style>
