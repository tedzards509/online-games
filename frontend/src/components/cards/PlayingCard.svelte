<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { MouseEventHandler } from 'svelte/elements';

  /** The shared 5:7 shell. Content may be any snippet, including SVG or images.
   * Supplying onclick makes a native button; display-only cards are non-focusable.
   * Width is a CSS length; omitted width inherits --arena-card-width.
   * `muted` shows an opaque, desaturated face (e.g. an illegal play);
   * `highlighted` marks a card such as the most recent play.
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
    muted = false,
    highlighted = false,
  }: {
    label: string;
    children: Snippet;
    width?: string;
    color?: string;
    background?: string;
    onclick?: MouseEventHandler<HTMLButtonElement>;
    disabled?: boolean;
    selected?: boolean;
    muted?: boolean;
    highlighted?: boolean;
  } = $props();
</script>

<svelte:element
  this={onclick ? 'button' : 'div'}
  class="playing-card"
  class:selected
  class:muted
  class:highlighted
  type={onclick ? 'button' : undefined}
  role={onclick ? undefined : 'img'}
  aria-label={label}
  aria-pressed={onclick ? selected : undefined}
  disabled={onclick ? disabled : undefined}
  {onclick}
  style:width
  style:--card-color={color}
  style:--card-background={background}
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
    border: var(--arena-card-line);
    border-radius: var(--arena-card-radius);
    background: var(--card-background, var(--arena-card-face));
    color: var(--card-color, var(--arena-card-dark));
    font-family: var(--arena-font);
    container-type: inline-size;
    transition:
      transform var(--arena-transition),
      box-shadow var(--arena-transition);
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
  button.playing-card:not(:disabled) {
    box-shadow: var(--arena-card-raised);
  }
  @media (hover: hover) {
    button.playing-card:not(:disabled):not(.selected):hover {
      transform: var(--arena-hover-shift);
    }
  }
  button.playing-card:disabled {
    cursor: not-allowed;
  }
  .playing-card.selected {
    transform: var(--arena-card-lift);
    background: var(--arena-card-selected);
    box-shadow: var(--arena-card-lifted);
  }
  .playing-card.muted {
    background: var(--arena-card-muted);
    color: var(--arena-on-card-muted);
  }
  .playing-card.highlighted {
    outline: 3px solid var(--arena-highlight);
    outline-offset: 3px;
  }
</style>
