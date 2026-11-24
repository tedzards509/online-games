<script lang="ts">
  import type { Snippet } from 'svelte';

  /** A labelled, non-overlapping row of cards. Long rows scroll horizontally
   * instead of shrinking cards; padding leaves room for lifted selections. */
  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="card-row" role="group" aria-label={label}>
  {@render children()}
</div>

<style>
  .card-row {
    display: flex;
    gap: 0.6rem;
    width: fit-content;
    max-width: 100%;
    margin-inline: auto;
    padding: 1rem 0.5rem 0.5rem;
    overflow-x: auto;
  }
</style>
