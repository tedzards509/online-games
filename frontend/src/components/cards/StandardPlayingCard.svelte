<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import { t } from '../../lib/i18n';
  import type { Card } from '../../lib/types';
  import SymbolCard from './SymbolCard.svelte';

  /** Standard suits use SVG so their shape does not depend on installed fonts. */
  let {
    card,
    label,
    ...shell
  }: Omit<
    ComponentProps<typeof SymbolCard>,
    'rank' | 'symbol' | 'symbolContent' | 'color' | 'label'
  > & {
    card: Card;
    label?: string;
  } = $props();
  const rank = $derived(
    ({ 11: 'J', 12: 'Q', 13: 'K', 14: 'A' } as Record<number, string>)[
      card.rank
    ] ?? String(card.rank),
  );
  const accessibleRank = $derived(
    card.rank > 10 ? $t(`ranks.${card.rank}`) : String(card.rank),
  );
  const red = $derived(card.suit === 'hearts' || card.suit === 'diamonds');
</script>

<SymbolCard
  {...shell}
  {rank}
  color={red ? 'var(--arena-card-red)' : 'var(--arena-card-dark)'}
  label={label ??
    $t('card', {
      values: { rank: accessibleRank, suit: $t(`suits.${card.suit}`) },
    })}
>
  {#snippet symbolContent()}
    <svg viewBox="0 0 32 32" fill="currentColor" aria-hidden="true">
      {#if card.suit === 'hearts'}
        <path d="M16 29 3 16C-5 6 9-2 16 8 23-2 37 6 29 16Z" />
      {:else if card.suit === 'diamonds'}
        <path d="m16 1 13 15-13 15L3 16Z" />
      {:else if card.suit === 'spades'}
        <path d="M16 1 3 14c-6 8 3 15 10 9l-3 8h12l-3-8c7 6 16-1 10-9Z" />
      {:else}
        <path
          d="M13 22c-13 9-18-12-5-11C3-4 29-4 24 11c13-1 8 20-5 11l3 9H10Z"
        />
      {/if}
    </svg>
  {/snippet}
</SymbolCard>

<style>
  svg {
    display: block;
    width: 100%;
    height: 100%;
    overflow: visible;
  }
</style>
