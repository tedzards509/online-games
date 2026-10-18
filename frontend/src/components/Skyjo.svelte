<script lang="ts">
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    you,
    names,
    move,
  }: {
    view: Schemas['SkyjoView'];
    actions: Action[];
    you: number;
    names: string[];
    move: (action: Action) => void;
  } = $props();
  const order = $derived([
    you,
    ...names.map((_, i) => i).filter((i) => i !== you),
  ]);
  function cellAction(player: number, index: number): Action | undefined {
    if (player !== you) return;
    return actions.find(
      (a) =>
        a.game === 'skyjo' &&
        (a.action.type === 'reveal' || a.action.type === 'replace') &&
        a.action.index === index,
    );
  }
  const hint = $derived(
    view.phase === 'setup'
      ? 'Reveal two'
      : view.phase === 'place'
        ? 'Replace a card'
        : view.phase === 'reveal'
          ? 'Reveal a card'
          : 'Draw a card',
  );
</script>

<div class="table-center">
  <span class="number-card" aria-label="Discard pile">{view.top ?? '—'}</span>
  {#if view.pending !== null && view.pending !== undefined}<span
      class="number-card pending"
      aria-label="Drawn card">{view.pending}</span
    >{/if}
  <span class="muted">{view.stock} in stock</span>
</div>
{#if actions.length}<p class="muted">{hint}</p>{/if}
<div class="actions">
  {#each actions as action}
    {#if action.game === 'skyjo' && action.action.type === 'draw'}
      <button onclick={() => move(action)}
        >{action.action.source === 'stock' ? 'Draw' : 'Take discard'}</button
      >
    {:else if action.game === 'skyjo' && action.action.type === 'discard'}
      <button onclick={() => move(action)}>Discard & reveal</button>
    {/if}
  {/each}
</div>
<div class="skyjo-tables">
  {#each order as player}
    <section aria-label={`${names[player]}'s grid`}>
      <h3>{names[player]}{player === you ? ' · you' : ''}</h3>
      <div class="skyjo-grid">
        {#each view.grids[player] as cell, index}
          {@const action = cellAction(player, index)}
          <button
            class="number-card"
            class:removed={cell.removed}
            class:back={!cell.removed && cell.value == null}
            disabled={!action}
            aria-label={`Card ${index + 1}: ${cell.removed ? 'removed' : (cell.value ?? 'hidden')}`}
            onclick={() => action && move(action)}
            >{cell.removed ? '—' : (cell.value ?? '?')}</button
          >
        {/each}
      </div>
    </section>
  {/each}
</div>
