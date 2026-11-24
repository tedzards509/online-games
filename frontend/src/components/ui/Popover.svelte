<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Native auto popover (Escape and outside clicks dismiss it), anchored below
   * the corner trigger buttons: `start` (left) or `end` (right). On narrow screens
   * it sits above the triggers, which move to the bottom corners. */
  let {
    id,
    label,
    placement,
    children,
  }: {
    id: string;
    label: string;
    placement: 'start' | 'end';
    children: Snippet;
  } = $props();
</script>

<div {id} popover="auto" role="dialog" class={placement} aria-label={label}>
  {@render children()}
</div>

<style>
  [popover] {
    position: fixed;
    margin: 0;
    max-height: calc(100dvh - 7rem);
    overflow: auto;
    padding: var(--arena-space-3);
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    box-shadow: var(--arena-overlay);
  }
  .start {
    inset: 5.25rem auto auto var(--arena-space-3);
    width: min(30rem, calc(100vw - 3rem));
  }
  .end {
    inset: 5.25rem var(--arena-space-3) auto auto;
    width: min(17rem, calc(100vw - 3rem));
  }
  @media (max-width: 40rem) {
    .start,
    .end {
      inset: auto auto calc(5rem + env(safe-area-inset-bottom))
        var(--arena-space-2);
      width: calc(100vw - 2rem);
    }
  }
</style>
