<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Own hands expose at least 44px per card; long hands scroll, not shrink.
   * Opponent fans are compact, decorative displays of public counts only.
   */
  let {
    count,
    compact = false,
    selected = -1,
    card,
  }: {
    count: number;
    compact?: boolean;
    selected?: number;
    card: Snippet<[number]>;
  } = $props();
</script>

<div class="fan" class:compact style:--count={Math.max(1, count)}>
  {#each Array(count) as _, index}
    {@const distance = index - (count - 1) / 2}
    <div
      class="fan-card"
      class:selected={selected === index}
      style:--index={index}
      style:transform={`translateY(${Math.abs(distance) * (compact ? 0.6 : 1.4) - (selected === index ? 14 : 0)}px) rotate(${distance * (compact ? 2 : 1.8)}deg)`}
    >
      {@render card(index)}
    </div>
  {/each}
</div>

<style>
  .fan {
    --fan-width: 5.25rem;
    --fan-step: 2.75rem;
    --arena-card-width: var(--fan-width);
    position: relative;
    width: calc(var(--fan-width) + (var(--count) - 1) * var(--fan-step));
    height: calc(var(--fan-width) * 1.4 + 2rem);
    margin: 1.5rem auto 0;
  }
  .fan-card {
    position: absolute;
    top: 0;
    left: calc(var(--index) * var(--fan-step));
    transform-origin: 50% 100%;
  }
  .fan-card.selected {
    z-index: 2;
  }
  .fan-card:focus-within {
    z-index: 3;
  }
  .compact {
    --fan-width: 2.125rem;
    --fan-step: 0.25rem;
    height: 3.75rem;
    margin-top: 0.5rem;
  }
  @media (max-width: 40rem) {
    .fan:not(.compact) {
      --fan-width: 4.5rem;
    }
  }
</style>
