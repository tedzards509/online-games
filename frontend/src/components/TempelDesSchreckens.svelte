<script lang="ts">
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    names,
    move,
    you,
    finished,
  }: {
    view: Schemas['TempelView'];
    actions: Action[];
    names: string[];
    move: (action: Action) => void;
    you: number;
    finished: boolean;
  } = $props();
  const available = $derived(
    actions.filter((a) => a.game === 'tempel_des_schreckens'),
  );
  function reveal(player: number, index: number): Action | undefined {
    return available.find(
      (a) =>
        a.action.type === 'reveal' &&
        a.action.player === player &&
        a.action.index === index,
    );
  }
</script>

<p>Round {view.round} / 4 · {view.round_reveals} / {names.length} revealed</p>
<p aria-label="Your role">
  You are {view.role === 'adventurer' ? 'an adventurer' : 'a guardian'}
</p>
<p aria-label="Your card counts">
  Your hidden cards: {view.hand_counts.gold} gold · {view.hand_counts.traps} traps
  · {view.hand_counts.empty} empty
</p>
<p aria-label="Revealed piles">
  Revealed: {view.revealed.gold} / {view.distribution.cards.gold} gold · {view
    .revealed.traps} / {view.distribution.cards.traps} traps · {view.revealed
    .empty} empty
</p>
<p>
  Role pool: {view.distribution.adventurers} adventurers · {view.distribution
    .guardians} guardians
</p>
<p>Key: {names[view.key]}</p>
{#if !finished}
  {#if view.phase === 'inspect'}<p>
      {view.ready[you]
        ? 'Waiting for other players'
        : 'Check your card counts, then ready'}
    </p>{/if}
  {#each available as action}
    {#if action.action.type === 'ready'}
      <button onclick={() => move(action)}>Ready</button>
    {/if}
  {/each}
{/if}
{#each names as name, player}
  <section aria-label={`${name}'s cards`}>
    <h3>
      {name}{view.phase === 'inspect' && view.ready[player] ? ' · ready' : ''}
    </h3>
    <div class="hand">
      {#each view.cards[player] as card, index}
        <button
          disabled={!reveal(player, index)}
          aria-label={`${name} card ${index + 1}: ${card ?? 'hidden'}`}
          onclick={() => {
            const action = reveal(player, index);
            if (action) move(action);
          }}>{card ?? '?'}</button
        >
      {/each}
    </div>
  </section>
{/each}
