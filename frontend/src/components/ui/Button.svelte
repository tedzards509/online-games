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
  <span class="content">{@render children()}</span>
</button>

<style>
  button {
    position: relative;
    isolation: isolate;
    overflow: hidden;

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

    transform: translateY(0) rotate(0);

    transition:
      background 180ms ease,
      border-color 180ms ease,
      transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1),
      box-shadow 180ms ease;
  }

  /* Primary: curved upward color sweep */
  button::before {
    content: '';
    position: absolute;
    z-index: -1;
    inset: 0;
    pointer-events: none;

    background: var(--arena-focus);

    clip-path: ellipse(100% 0% at 85% 100%);

    transition: clip-path 420ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .content {
    position: relative;
    z-index: 1;
  }

  /* Secondary variant */
  button.secondary {
    background: var(--arena-surface);
    color: var(--arena-text);
    border-color: var(--arena-border);
  }

  button.secondary::before {
    display: none;
  }

  /* Hover interactions */
  @media (hover: hover) {
    /* Primary: tilt left */
    button:not(.secondary):hover:not(:disabled) {
      transform: translateY(-3px) rotate(-1deg);
      box-shadow: 0 6px 0 rgba(0, 0, 0, 0.12);
    }

    /* Primary: expanding upward sweep */
    button:not(.secondary):hover:not(:disabled)::before {
      clip-path: ellipse(150% 220% at 85% 100%);
    }

    /* Secondary: tilt right, no sweep */
    button.secondary:hover:not(:disabled) {
      transform: translateY(-3px) rotate(1deg);
      background: var(--arena-input);
      border-color: var(--arena-text);
      box-shadow: 0 6px 0 rgba(0, 0, 0, 0.12);
    }
  }

  /* Tactile press feedback */
  button:active:not(:disabled) {
    transform: translateY(2px) scale(0.97);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.12);
    transition-duration: 80ms;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button:focus-visible {
    outline: 3px solid var(--arena-focus);
    outline-offset: 3px;
  }

  /* Accessibility */
  @media (prefers-reduced-motion: reduce) {
    button,
    button::before {
      transition: none;
    }

    button:hover:not(:disabled),
    button:active:not(:disabled) {
      transform: none;
    }
  }
</style>
