<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  /** The shared playing surface; centres its children and wraps them in rows. */
  let {
    children,
    ...attributes
  }: Omit<HTMLAttributes<HTMLDivElement>, 'class'> & { children: Snippet } =
    $props();
</script>

<div {...attributes} class="table">{@render children()}</div>

<style>
  .table {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--arena-space-3);
    min-height: 12rem;
    padding: var(--arena-space-3);
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-table);
    color: var(--arena-on-table);
  }
</style>
