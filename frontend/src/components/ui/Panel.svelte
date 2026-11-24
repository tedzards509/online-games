<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  /** Outlined surface that stacks its children; the parent owns placement. */
  let {
    children,
    ...attributes
  }: Omit<HTMLAttributes<HTMLElement>, 'class'> & { children: Snippet } =
    $props();
</script>

<section {...attributes} class="panel">{@render children()}</section>

<style>
  .panel {
    display: grid;
    gap: var(--arena-space-2);
    min-width: 0;
    padding: var(--arena-space-3);
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
  }
</style>
