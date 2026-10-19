<script lang="ts">
  import PlayingCard from './PlayingCard.svelte';
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    you,
    names,
    move,
  }: {
    view: Schemas['ShitheadView'];
    actions: Action[];
    you: number;
    names: string[];
    move: (action: Action) => void;
  } = $props();
  let selected = $state<number[]>([]);
  const source = $derived(view.hand.length ? view.hand : view.players[you].up);
  const indices = $derived([...selected].sort((a, b) => a - b));
  const playable = $derived(
    actions.find(
      (a) =>
        a.game === 'shithead' &&
        (a.action.type === 'play' || a.action.type === 'select_face_up') &&
        a.action.indices.length === indices.length &&
        a.action.indices.every((index, i) => index === indices[i]),
    ),
  );
  $effect(() => {
    view;
    actions;
    selected = [];
  });
  function toggle(index: number): void {
    selected = selected.includes(index)
      ? selected.filter((i) => i !== index)
      : [...selected, index];
  }
</script>

<div class="opponents">
  {#each view.players as player, index}
    {#if index !== you}
      <div class="player-cards">
        <span>{names[index]} · {player.count} in hand · {player.down} down</span
        >
        <div class="hand small">
          {#each player.up as card}<PlayingCard {card} disabled />{/each}
        </div>
      </div>
    {/if}
  {/each}
</div>
<div class="table-center">
  {#if view.top}<PlayingCard card={view.top} disabled />{:else}<span
      class="empty-card">—</span
    >{/if}
  <span class="muted">{view.pile} in pile · {view.stock} in stock</span>
  {#if view.top?.rank === 3}
    <span>3 is transparent</span>
    {#if view.effective_top}<PlayingCard
        card={view.effective_top}
        disabled
      />{/if}
  {/if}
</div>
{#if view.setup}<p class="muted">
    {actions.length
      ? 'Choose three face-up cards'
      : 'Waiting for other players'}
  </p>{/if}
<div
  class="hand"
  aria-label={view.hand.length ? 'Your hand' : 'Your face-up cards'}
>
  {#each source as card, index}
    <PlayingCard
      {card}
      selected={selected.includes(index)}
      disabled={!actions.some(
        (a) =>
          a.game === 'shithead' &&
          (a.action.type === 'play' || a.action.type === 'select_face_up') &&
          a.action.indices.includes(index),
      )}
      onclick={() => toggle(index)}
    />
  {/each}
</div>
<div class="hand" aria-label="Your face-down cards">
  {#each Array(view.players[you].down) as _, index}
    {@const action = actions.find(
      (a) =>
        a.game === 'shithead' &&
        a.action.type === 'blind' &&
        a.action.index === index,
    )}
    <PlayingCard
      disabled={!action}
      label={`Face-down card ${index + 1}`}
      onclick={() => action && move(action)}
    />
  {/each}
</div>
{#if view.hand.length}
  <div class="hand small" aria-label="Your face-up cards">
    {#each view.players[you].up as card}<PlayingCard {card} disabled />{/each}
  </div>
{/if}
<div class="actions">
  {#if source.length}<button
      disabled={!playable}
      onclick={() => playable && move(playable)}
      >{view.setup ? 'Set face-up cards' : 'Play'}</button
    >{/if}
  {#each actions as action}
    {#if action.game === 'shithead' && action.action.type === 'pickup'}<button
        onclick={() => move(action)}>Pick up</button
      >{/if}
  {/each}
</div>
