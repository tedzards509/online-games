<script lang="ts">
  import type { Action, Schemas } from '../lib/types';
  let {
    view,
    actions,
    names,
    move,
    finished,
  }: {
    view: Schemas['CrewView'];
    actions: Action[];
    names: string[];
    move: (action: Action) => void;
    finished: boolean;
  } = $props();
  const crewActions = $derived(actions.filter((a) => a.game === 'the_crew'));
  function label(card: Schemas['CrewCard']): string {
    return `${card.color} ${card.number}`;
  }
  function play(index: number): Action | undefined {
    return crewActions.find(
      (a) => a.action.type === 'play' && a.action.index === index,
    );
  }
</script>

{#if !finished}
  <p>
    {view.awaiting_result
      ? 'Waiting for host to record the result'
      : `${names[view.next_player]} ${view.trick.length ? 'plays next' : 'leads'}`}
  </p>
{/if}
<section aria-label="Current trick">
  {#each view.trick as played}<p>
      {names[played.player]}: {label(played.card)}
    </p>{/each}
</section>
{#if view.last_trick}
  <details>
    <summary>Last trick · {names[view.last_trick.winner]}</summary>
    {#each view.last_trick.cards as played}<p>
        {names[played.player]}: {label(played.card)}
      </p>{/each}
  </details>
{/if}
<section aria-label="Crew">
  {#each names as name, player}
    <p>
      {name} · {view.counts[player]} cards · {view.tricks_won[player]} tricks
      {#if view.communications[player]}
        · {label(view.communications[player].card)}: {view.communications[
          player
        ].declaration}
      {/if}
    </p>
  {/each}
</section>
<section aria-label="Your hand" class="hand">
  {#each view.hand as card, index}
    <button
      disabled={!play(index)}
      onclick={() => {
        const action = play(index);
        if (action) move(action);
      }}
      aria-label={`Play ${label(card)}`}>{label(card)}</button
    >
  {/each}
</section>
{#if crewActions.some((a) => a.action.type === 'communicate')}
  <details>
    <summary>Communicate a card</summary>
    {#each crewActions as action}
      {#if action.action.type === 'communicate'}
        <button onclick={() => move(action)}
          >Communicate {label(view.hand[action.action.index])} as {action.action
            .declaration}</button
        >
      {/if}
    {/each}
  </details>
{/if}
