<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';
  import { t } from '../../lib/i18n';
  import Button from '../ui/Button.svelte';

  /** The viewer's hand: one labelled row that overlaps cards once it grows and
   * scrolls horizontally beyond that. `card` renders each item with its index in
   * `items`, so actions keep their indices whatever the display order. With
   * `order`, a toggle shows the hand sorted by that comparison. */
  let {
    label,
    items,
    order,
    card,
  }: {
    label: string;
    items: T[];
    order?: (a: T, b: T) => number;
    card: Snippet<[T, number]>;
  } = $props();
  let sorted = $state(false);
  const entries = $derived.by(() => {
    const entries = items.map((item, index) => ({ item, index }));
    if (sorted && order) entries.sort((a, b) => order(a.item, b.item));
    return entries;
  });
</script>

<div class="hand">
  <div class="cards" role="group" aria-label={label}>
    {#each entries as { item, index } (index)}
      <div class="slot">{@render card(item, index)}</div>
    {/each}
  </div>
  {#if order}
    <Button
      variant="quiet"
      size="small"
      square
      aria-label={$t('common.sortByValue')}
      aria-pressed={sorted}
      onclick={() => (sorted = !sorted)}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M4 18h4M4 12h9M4 6h14M18 11v8m-3-3 3 3 3-3"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </Button>
  {/if}
</div>

<style>
  .hand {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--arena-space-2);
    min-width: 0;
  }
  .hand > :global(button) {
    flex: none;
  }
  .cards {
    display: flex;
    gap: 0.6rem;
    min-width: 0;
    /* Room above for lifted selections and beside for the raised shadow. */
    padding: 1rem 0.5rem 0.5rem;
    overflow-x: auto;
    scroll-snap-type: x proximity;
    scroll-padding-inline: 0.5rem;
    scrollbar-width: thin;
  }
  .slot {
    position: relative;
    flex: none;
    scroll-snap-align: start;
  }
  /* Once a row would no longer fit (13 cards wide, 5 narrow), each card covers
     the right part of the one before; the top-left index stays readable. */
  .cards:has(> .slot:nth-child(13)) {
    gap: 0;
  }
  .cards:has(> .slot:nth-child(13)) > .slot + .slot {
    margin-left: calc(var(--arena-card-width) * -0.4);
  }
  @media (max-width: 40rem) {
    .cards:has(> .slot:nth-child(5)) {
      gap: 0;
    }
    .cards:has(> .slot:nth-child(5)) > .slot + .slot {
      margin-left: calc(var(--arena-card-width) * -0.4);
    }
  }
  /* Selected cards always sit above their overlapping neighbours. */
  .slot:has(:global([aria-pressed='true'])) {
    z-index: 1;
  }
</style>
