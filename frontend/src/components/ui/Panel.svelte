<script lang="ts">
  import type { Snippet } from 'svelte';
  let { children }: { children: Snippet } = $props();
</script>

<div class="panel">{@render children()}</div>

<style>
  .panel {
    height: 100%;
    padding: var(--arena-space-3);
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
  }
</style>
