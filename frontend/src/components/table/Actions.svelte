<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Centred, wrapping row of game command buttons. */
  let { label, children }: { label?: string; children: Snippet } = $props();
</script>

<div class="actions" role={label ? 'group' : undefined} aria-label={label}>
  {@render children()}
</div>

<style>
  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.75rem;
  }
</style>
