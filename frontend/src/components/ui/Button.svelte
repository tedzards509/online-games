<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    variant = 'primary',
    children,
    ...attributes
  }: HTMLButtonAttributes & {
    variant?: 'primary' | 'secondary';
    children: Snippet;
  } = $props();
</script>

<button type="button" {...attributes} class:secondary={variant === 'secondary'}>
  {@render children()}
</button>

<style>
  button {
    min-height: var(--arena-control-height);
    padding: 0.75rem 1.25rem;
    border: 1px solid transparent;
    border-radius: var(--arena-control-radius);
    font-family: var(--arena-font);
    font-size: 1rem;
    font-weight: 750;
    line-height: 1.3;
    background: var(--arena-primary);
    color: var(--arena-on-primary);
    box-shadow: var(--arena-shadow);
    cursor: pointer;
    transition: background var(--arena-transition);
  }
  button.secondary {
    background: var(--arena-surface);
    color: var(--arena-text);
    border-color: var(--arena-border);
  }
  button:hover:not(:disabled) {
    background: var(--arena-primary-hover);
    border-color: transparent;
  }
  button.secondary:hover:not(:disabled) {
    background: var(--arena-input);
    border-color: var(--arena-text);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button:focus-visible {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }
  @media (prefers-reduced-motion: reduce) {
    button {
      transition: none;
    }
  }
</style>
