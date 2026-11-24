<script lang="ts">
  import type { Snippet } from 'svelte';

  /** A square board position (cell, pit) that fills its grid track. Raised and
   * pressable while it accepts a move; flat otherwise. Font size is inherited. */
  let {
    label,
    move,
    round = false,
    children,
  }: {
    label: string;
    /** The move this space submits, if it is currently legal. */
    move?: () => void;
    round?: boolean;
    children?: Snippet;
  } = $props();
</script>

<button
  class="pressable"
  class:round
  aria-label={label}
  disabled={!move}
  onclick={() => move?.()}
>
  {@render children?.()}
</button>

<style>
  button {
    display: grid;
    place-items: center;
    width: 100%;
    min-width: 2.75rem;
    aspect-ratio: 1;
    padding: 0;
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    font-weight: 800;
    line-height: 1;
    transition:
      transform var(--arena-transition),
      box-shadow var(--arena-transition);
  }
  .round {
    border-radius: 50%;
  }
  button:not(:disabled) {
    cursor: pointer;
    box-shadow: var(--arena-raised);
  }
  @media (hover: hover) {
    button:not(:disabled):hover {
      transform: var(--arena-hover-shift);
      box-shadow: var(--arena-raised-hover);
    }
  }
  button:not(:disabled):active {
    transform: var(--arena-pressed-shift);
    box-shadow: var(--arena-pressed);
  }
</style>
