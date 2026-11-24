<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  /** Native button with the theme's raised/pressed treatment.
   * `square` sizes an icon-only button; give it an aria-label. */
  let {
    variant = 'primary',
    size = 'regular',
    square = false,
    children,
    ...attributes
  }: Omit<HTMLButtonAttributes, 'class'> & {
    variant?: 'primary' | 'secondary' | 'quiet';
    size?: 'small' | 'regular' | 'large';
    square?: boolean;
    children: Snippet;
  } = $props();
</script>

<button
  type="button"
  {...attributes}
  class="pressable {variant} {size}"
  class:square
>
  {@render children()}
</button>

<style>
  button {
    min-height: var(--arena-control-height);
    padding: 0.5rem 1.25rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--arena-space-1);
    border: var(--arena-line);
    border-radius: var(--arena-control-radius);
    font-family: var(--arena-font);
    font-size: 1rem;
    font-weight: 800;
    line-height: 1.2;
    cursor: pointer;
    box-shadow: var(--arena-raised);
    transition:
      transform var(--arena-transition),
      box-shadow var(--arena-transition);
  }
  .primary {
    background: var(--arena-primary);
    color: var(--arena-on-primary);
  }
  .secondary {
    background: var(--arena-secondary);
    color: var(--arena-on-secondary);
  }
  .quiet {
    background: var(--arena-surface);
    color: var(--arena-text);
  }
  /* Still a 44px touch target. */
  .small {
    min-height: 2.75rem;
    padding: 0.25rem 0.75rem;
    font-size: 0.9rem;
  }
  .large {
    min-height: 3.75rem;
    font-size: 1.2rem;
  }
  .square {
    width: var(--arena-control-height);
    padding: 0.6rem;
  }
  .small.square {
    width: 2.75rem;
  }
  .square :global(svg) {
    width: 100%;
    height: 100%;
  }
  @media (hover: hover) {
    button:hover:not(:disabled) {
      transform: var(--arena-hover-shift);
      box-shadow: var(--arena-raised-hover);
    }
  }
  button:active:not(:disabled) {
    transform: var(--arena-pressed-shift);
    box-shadow: var(--arena-pressed);
  }
  button:disabled {
    background: var(--arena-disabled);
    color: var(--arena-on-disabled);
    box-shadow: none;
    cursor: not-allowed;
  }
</style>
