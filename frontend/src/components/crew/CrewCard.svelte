<script lang="ts">
  import type { ComponentProps } from 'svelte';
  import SymbolCard from '../cards/SymbolCard.svelte';
  import type { Schemas } from '../../lib/types';
  import { t } from '../../lib/i18n';
  import { crewColors, crewInk } from './cards';

  let {
    card,
    label,
    muted = false,
    selected = false,
    ...shell
  }: Omit<
    ComponentProps<typeof SymbolCard>,
    'rank' | 'symbol' | 'symbolContent' | 'label' | 'color' | 'background'
  > & {
    card: Schemas['CrewCard'];
    label?: string;
    muted?: boolean;
  } = $props();
</script>

<span class="crew-card" class:selected data-muted={muted}>
  <SymbolCard
    {...shell}
    rank={card.number}
    {selected}
    color={card.color === 'black' && !muted ? '#fff8e7' : crewInk}
    background={muted
      ? `color-mix(in srgb, ${crewColors[card.color]} ${card.color === 'black' ? 40 : 55}%, var(--arena-canvas))`
      : crewColors[card.color]}
    label={label ??
      $t('crew.card', {
        values: {
          color: $t(`crew.colors.${card.color}`),
          number: String(card.number),
        },
      })}
  >
    {#snippet symbolContent()}
      <svg
        viewBox="0 0 32 32"
        fill="none"
        stroke="currentColor"
        stroke-width="2.25"
        stroke-linejoin="round"
        stroke-linecap="round"
        aria-hidden="true"
      >
        {#if card.color === 'blue'}
          <circle cx="16" cy="16" r="14" />
        {:else if card.color === 'green'}
          <path d="M16 2 31 29H1Z" />
        {:else if card.color === 'pink'}
          <rect x="3" y="3" width="26" height="26" rx="1" />
        {:else if card.color === 'yellow'}
          <path d="M5 5 27 27M27 5 5 27" />
        {:else}
          <path
            d="M3 17a13 13 0 0 1 26 0c-2 2-4 2-6 0-2 2-5 2-7 0-2 2-5 2-7 0-2 2-4 2-6 0Z"
          />
          <path
            d="M8 19c-4 4 4 6 0 10M16 19c-4 4 4 7 0 11M24 19c-4 4 4 6 0 10"
          />
        {/if}
      </svg>
    {/snippet}
  </SymbolCard>
</span>

<style>
  .crew-card {
    display: inline-flex;
    flex-shrink: 0;
    vertical-align: top;
    border-radius: var(--arena-card-radius);
  }
  .crew-card :global(.playing-card) {
    border-color: #ffffff55;
    box-shadow: 0 2px 4px #15291d18;
  }
  .crew-card.selected :global(.playing-card) {
    border-color: var(--arena-text);
    box-shadow: 0 0 0 2px var(--arena-text);
  }
  span.crew-card :global(button.playing-card:disabled) {
    opacity: 1;
  }
  svg {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
