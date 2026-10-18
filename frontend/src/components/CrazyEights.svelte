<script lang="ts">
  import PlayingCard from './PlayingCard.svelte';
  import { suits, type Action, type Schemas } from '../lib/types';
  let {
    view,
    actions,
    move,
  }: {
    view: Schemas['CrazyEightsView'];
    actions: Action[];
    move: (action: Action) => void;
  } = $props();
  let wild = $state<number | null>(null);
  const choices = $derived(
    actions.filter(
      (a) =>
        a.game === 'crazy_eights' &&
        a.action.type === 'play' &&
        a.action.index === wild,
    ),
  );
  $effect(() => {
    if (!choices.length) wild = null;
  });
  function play(index: number): void {
    const legal = actions.find(
      (a) =>
        a.game === 'crazy_eights' &&
        a.action.type === 'play' &&
        a.action.index === index,
    );
    if (!legal) return;
    if (view.hand[index].rank === 8) wild = index;
    else move(legal);
  }
</script>

<div class="table-center">
  <PlayingCard card={view.top} disabled />
  <span aria-label={`Current suit: ${view.suit}`}>{suits[view.suit]}</span>
  <span class="muted">{view.stock} in stock</span>
</div>
<div class="hand" aria-label="Your hand">
  {#each view.hand as card, index}
    <PlayingCard
      {card}
      selected={wild === index}
      disabled={!actions.some(
        (a) =>
          a.game === 'crazy_eights' &&
          a.action.type === 'play' &&
          a.action.index === index,
      )}
      onclick={() => play(index)}
    />
  {/each}
</div>
{#if choices.length}
  <div class="actions" aria-label="Choose suit">
    {#each choices as choice}
      {#if choice.game === 'crazy_eights' && choice.action.type === 'play' && choice.action.suit}
        <button
          aria-label={`Choose ${choice.action.suit}`}
          onclick={() => {
            move(choice);
            wild = null;
          }}>{suits[choice.action.suit]}</button
        >
      {/if}
    {/each}
  </div>
{/if}
{#each actions as action}
  {#if action.game === 'crazy_eights' && action.action.type === 'draw'}
    <button onclick={() => move(action)}
      >{view.stock === 0 ? 'Draw / pass' : 'Draw'}</button
    >
  {/if}
{/each}
