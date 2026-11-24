<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../../lib/i18n';
  import Avatar from './Avatar.svelte';

  /** The viewer's seat, kept at the bottom of the screen: an optional hand
   * above a bar with identity, public details, a short hint, and the action
   * buttons (`controls`; put the primary action last). `attention` marks that
   * the viewer must act now (also exposed as data-active). Render it last in a
   * Board, only for seated viewers. */
  let {
    name,
    colorIndex,
    attention = false,
    hint,
    hand,
    details,
    controls,
  }: {
    name: string;
    colorIndex: number;
    attention?: boolean;
    hint?: string;
    hand?: Snippet;
    details?: Snippet;
    controls?: Snippet;
  } = $props();
</script>

<section class="dock" aria-label={$t('common.yourSeat')}>
  {#if hand}{@render hand()}{/if}
  <div class="bar" class:attention data-active={attention}>
    <div class="identity">
      <Avatar {name} {colorIndex} />
      <span class="name"
        >{name}<span class="you">{` · ${$t('common.you')}`}</span></span
      >
      {#if details}<span class="details">{@render details()}</span>{/if}
    </div>
    <p class="hint" aria-live="polite">{hint ?? ''}</p>
    {#if controls}<div class="controls">{@render controls()}</div>{/if}
  </div>
</section>

<style>
  .dock {
    position: sticky;
    bottom: 0;
    z-index: 1;
    display: grid;
    gap: var(--arena-space-1);
    padding-block: var(--arena-space-1) var(--arena-space-2);
    background: var(--arena-canvas);
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--arena-space-2) var(--arena-space-3);
    min-height: calc(var(--arena-control-height) + 1.5rem);
    padding: 0.6rem 0.75rem;
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
  }
  /* Not pressable, so no shadow: attention is the highlight fill plus a crisp
     ring that slowly widens and fades. */
  .attention {
    background: var(--arena-highlight);
    color: var(--arena-on-highlight);
    animation: attention 2.4s ease-out infinite;
  }
  @keyframes attention {
    from {
      box-shadow: 0 0 0 0 var(--arena-highlight);
    }
    to {
      box-shadow: 0 0 0 0.6rem transparent;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .attention {
      animation: none;
      box-shadow: 0 0 0 4px var(--arena-highlight);
    }
  }
  .identity {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem 0.65rem;
    min-width: 0;
    font-weight: 700;
  }
  .name {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .you,
  .details {
    font-weight: 500;
  }
  .details {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem 0.75rem;
  }
  .hint {
    flex: 1 1 12rem;
    font-weight: 700;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-left: auto;
  }
  @media (max-width: 40rem) {
    /* Clears the corner buttons that sit at the bottom on narrow screens. */
    .dock {
      bottom: calc(var(--arena-control-height) + var(--arena-space-2));
    }
    .hint {
      flex-basis: 100%;
    }
    .hint:empty {
      display: none;
    }
    .controls {
      flex-basis: 100%;
    }
    .controls > :global(*) {
      flex: 1 1 auto;
    }
  }
</style>
