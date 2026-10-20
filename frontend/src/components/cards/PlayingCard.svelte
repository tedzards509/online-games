<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { MouseEventHandler } from 'svelte/elements';

  /** The shared 5:7 shell. Content may be any snippet, including SVG or images.
   * Supplying onclick makes a native button; display-only cards are non-focusable.
   * Width is a CSS length; omitted width inherits --arena-card-width.
   */
  let {
    label,
    children,
    width,
    color,
    background,
    onclick,
    disabled = false,
    selected,
  }: {
    label: string;
    children: Snippet;
    width?: string;
    color?: string;
    background?: string;
    onclick?: MouseEventHandler<HTMLButtonElement>;
    disabled?: boolean;
    selected?: boolean;
  } = $props();
</script>

<svelte:element
  this={onclick ? 'button' : 'div'}
  class="playing-card"
  class:selected
  type={onclick ? 'button' : undefined}
  role={onclick ? undefined : 'img'}
  aria-label={label}
  aria-pressed={onclick ? selected : undefined}
  disabled={onclick ? disabled : undefined}
  {onclick}
  style:width
  style:color
  style:background
>
  <span class="content" aria-hidden="true">{@render children()}</span>
</svelte:element>

<style>
  .playing-card {
    position: relative;
    display: inline-grid;
    place-items: stretch;
    flex: 0 0 auto;
    width: var(--arena-card-width);
    aspect-ratio: 5 / 7;
    padding: 0;
    overflow: hidden;
    border: 2px solid var(--arena-card-face);
    border-radius: var(--arena-card-radius);
    background: var(--arena-card-face);
    color: var(--arena-card-dark);
    box-shadow: var(--arena-card-shadow);
    font-family: var(--arena-font);
    container-type: inline-size;
  }
  .content {
    display: grid;
    place-items: center;
    min-width: 0;
    min-height: 0;
  }
  button.playing-card {
    cursor: pointer;
  }
  button.playing-card:hover:not(:disabled) {
    border-color: var(--arena-border);
  }
  button.playing-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .playing-card.selected {
    border-color: var(--arena-focus);
  }
  button.playing-card:focus-visible {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }
</style>
