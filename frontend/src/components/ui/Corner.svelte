<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Fixes its children to a top corner, or a bottom corner on narrow screens,
   * where thumbs reach. Pairs with Popover of the same placement. */
  let {
    placement,
    children,
  }: { placement: 'start' | 'end'; children: Snippet } = $props();
</script>

<div class={placement}>{@render children()}</div>

<style>
  div {
    position: fixed;
    top: var(--arena-space-3);
    z-index: 10;
  }
  .start {
    left: max(var(--arena-space-3), env(safe-area-inset-left));
  }
  .end {
    right: max(var(--arena-space-3), env(safe-area-inset-right));
  }
  @media (max-width: 40rem) {
    div {
      top: auto;
      bottom: max(var(--arena-space-2), env(safe-area-inset-bottom));
    }
    .start {
      left: max(var(--arena-space-2), env(safe-area-inset-left));
    }
    .end {
      right: max(var(--arena-space-2), env(safe-area-inset-right));
    }
  }
</style>
