<script lang="ts">
  import type { ComponentProps, Snippet } from 'svelte';
  import PlayingCard from './PlayingCard.svelte';

  /** Configurable rank/letter and symbol, with mirrored corner indices.
   * A symbol snippet replaces the text symbol in all three positions.
   * Shell props (label, colors, size, interaction) pass through unchanged.
   */
  let {
    rank,
    symbol,
    symbolContent,
    ...shell
  }: Omit<ComponentProps<typeof PlayingCard>, 'children'> & {
    rank: string | number;
    symbol?: string;
    symbolContent?: Snippet;
  } = $props();
</script>

{#snippet mark()}
  {#if symbolContent}{@render symbolContent()}{:else}{symbol ?? ''}{/if}
{/snippet}

<PlayingCard {...shell}>
  <span class="corner top"
    ><span>{rank}</span><span class="small-symbol">{@render mark()}</span></span
  >
  <span class="center">{@render mark()}</span>
  <span class="corner bottom"
    ><span>{rank}</span><span class="small-symbol">{@render mark()}</span></span
  >
</PlayingCard>

<style>
  .corner {
    position: absolute;
    display: grid;
    justify-items: center;
    gap: 2cqw;
    font-size: 21cqw;
    font-weight: 800;
    line-height: 1;
  }
  .top {
    top: 8cqw;
    left: 9cqw;
  }
  .bottom {
    bottom: 8cqw;
    right: 9cqw;
    transform: rotate(180deg);
  }
  .small-symbol {
    display: grid;
    place-items: center;
    width: 17cqw;
    height: 17cqw;
    font-size: 17cqw;
  }
  .center {
    display: grid;
    place-items: center;
    width: 45cqw;
    height: 45cqw;
    font-size: 45cqw;
    line-height: 1;
  }
</style>
