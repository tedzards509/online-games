<script lang="ts">
  import type { Schemas } from '../../lib/types';
  import CardBack from '../cards/CardBack.svelte';
  import PlayingCard from '../cards/PlayingCard.svelte';

  /** A chamber card: face-down (null) or revealed gold, trap, or empty. */
  let {
    card,
    label,
    disabled,
    onclick,
  }: {
    card: Schemas['TempelCard'] | null;
    label: string;
    disabled: boolean;
    onclick: () => void;
  } = $props();
</script>

{#if card === null}
  <CardBack {label} {disabled} {onclick} />
{:else}
  <PlayingCard
    {label}
    {disabled}
    {onclick}
    background={card === 'gold' ? 'var(--arena-highlight)' : undefined}
    color={card === 'trap'
      ? 'var(--arena-card-red)'
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
        <path d="M3 27 9 9l4 10 3-14 3 14 4-10 6 18Z" fill="currentColor" />
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
