<script lang="ts">
  import type { Schemas } from '../../lib/types';
  import CardBack from '../cards/CardBack.svelte';
  import PlayingCard from '../cards/PlayingCard.svelte';

  /** A chamber card: face-down (null) or revealed gold, trap, or empty.
   * Without onclick it is display-only. */
  let {
    card,
    label,
    disabled = false,
    muted = false,
    width,
    onclick,
  }: {
    card: Schemas['TempelCard'] | null;
    label: string;
    disabled?: boolean;
    muted?: boolean;
    width?: string;
    onclick?: () => void;
  } = $props();
</script>

{#if card === null}
  <CardBack {label} {disabled} {width} {onclick} />
{:else}
  <PlayingCard
    {label}
    {disabled}
    {muted}
    {width}
    {onclick}
    background={card === 'gold'
      ? 'var(--arena-highlight)'
      : card === 'trap'
        ? 'var(--arena-card-red)'
        : 'var(--arena-card-muted)'}
    color={card === 'trap'
      ? 'var(--arena-card-face)'
      : card === 'empty'
        ? 'var(--arena-on-card-muted)'
        : undefined}
  >
    <svg viewBox="0 0 32 32" aria-hidden="true">
      {#if card === 'gold'}
        <circle
          cx="16"
          cy="16"
          r="11"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
        />
        <circle cx="16" cy="16" r="5" fill="currentColor" />
      {:else if card === 'trap'}
        <g
          stroke="var(--arena-card-ink)"
          stroke-width="2"
          stroke-linejoin="round"
        >
          <path
            d="M16 3c1.5 5 8.5 8 8.5 16.5a8.5 8.5 0 0 1-17 0c0-4.5 2.5-6.5 3.5-10 2 2.5 2.5 4 2.5 6 1.5-3.5 2.5-7 2.5-12.5Z"
            fill="currentColor"
          />
          <path
            d="M16 16.5c1 2.5 4 3.5 4 7a4 4 0 0 1-8 0c0-2.5 2.5-3.5 4-7Z"
            fill="var(--arena-highlight)"
          />
        </g>
      {:else}
        <circle
          cx="16"
          cy="16"
          r="10"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-dasharray="4 4"
        />
      {/if}
    </svg>
  </PlayingCard>
{/if}

<style>
  svg {
    width: 60%;
  }
</style>
