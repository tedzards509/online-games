<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import SymbolCard from '../cards/SymbolCard.svelte';
  import type { Schemas } from '../../lib/types';
  import { t } from '../../lib/i18n';
  import { crewColors } from './cards';

  let {
    card,
    label,
    ...shell
  }: Omit<
    ComponentProps<typeof SymbolCard>,
    'rank' | 'symbol' | 'symbolContent' | 'label' | 'color'
  > & {
    card: Schemas['CrewCard'];
    label?: string;
  } = $props();
</script>

<SymbolCard
  {...shell}
  rank={card.number}
  color="var(--arena-text)"
  label={label ??
    $t('crew.card', {
      values: {
        color: $t(`crew.colors.${card.color}`),
        number: String(card.number),
      },
    })}
>
  {#snippet symbolContent()}
    <svg viewBox="0 0 32 32" fill={crewColors[card.color]} aria-hidden="true">
      {#if card.color === 'blue'}
        <circle cx="16" cy="16" r="14" />
      {:else if card.color === 'green'}
        <path d="M16 2 31 29H1Z" />
      {:else if card.color === 'pink'}
        <rect x="3" y="3" width="26" height="26" rx="1" />
      {:else if card.color === 'yellow'}
        <path d="M11 1h10v10h10v10H21v10H11V21H1V11h10Z" />
      {:else}
        <path
          d="M16 1c-6 5-8 12-6 21h12c2-9 0-16-6-21ZM9 14l-7 12 8-3Zm14 0 7 12-8-3ZM12 25h8l-4 7Z"
        />
        <circle cx="16" cy="12" r="3" fill="var(--arena-card-face)" />
      {/if}
    </svg>
  {/snippet}
</SymbolCard>

<style>
  svg {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
