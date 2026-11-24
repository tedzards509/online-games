<script lang="ts">
  import { t } from '../../lib/i18n';
  import CardBack from '../cards/CardBack.svelte';
  let { count, name }: { count: number; name: string } = $props();
</script>

<div
  class="pile"
  role="img"
  aria-label={$t('crew.tricksOf', { values: { name, count: String(count) } })}
>
  <div aria-hidden="true" class="pile-face">
    {#if count}<CardBack />{/if}
    <span class="count">{count}</span>
  </div>
</div>

<style>
  .pile {
    width: var(--arena-card-width);
  }
  .pile-face {
    position: relative;
    aspect-ratio: 5 / 7;
    border: 2px dashed var(--arena-text-muted);
    border-radius: var(--arena-card-radius);
  }
  /* The hard shadow behind the back suggests a stack. */
  .pile-face:has(:global(.playing-card)) {
    border: 0;
    border-radius: var(--arena-card-radius);
    box-shadow: var(--arena-card-raised);
  }
  .count {
    position: absolute;
    inset: 20% auto auto 50%;
    transform: translateX(-50%);
    min-width: 1.5rem;
    height: 1.5rem;
    padding: 0 0.25rem;
    display: grid;
    place-items: center;
    border: var(--arena-line);
    border-radius: 50%;
    background: var(--arena-surface);
    color: var(--arena-text);
    font-size: 0.8rem;
    font-weight: 800;
  }
</style>
