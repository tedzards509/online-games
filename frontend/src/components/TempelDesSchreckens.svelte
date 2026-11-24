<script lang="ts">
  import type { Action, BoardProps, Schemas } from '../lib/types';
  import Button from './ui/Button.svelte';
  import Notice from './ui/Notice.svelte';
  import Actions from './table/Actions.svelte';
  import Board from './table/Board.svelte';
  import CardRow from './table/CardRow.svelte';
  import PlayerBar from './table/PlayerBar.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import TempelCard from './tempel/TempelCard.svelte';

  let {
    view,
    actions,
    names,
    move,
    you,
    active,
    finished,
  }: BoardProps<Schemas['TempelView']> = $props();
  const available = $derived(
    actions.filter((a) => a.game === 'tempel_des_schreckens'),
  );
  const ready = $derived(available.find((a) => a.action.type === 'ready'));
  function reveal(player: number, index: number): Action | undefined {
    return available.find(
      (a) =>
        a.action.type === 'reveal' &&
        a.action.player === player &&
        a.action.index === index,
    );
  }
</script>

<Board>
  <TableSurface aria-label="Revealed piles">
    <div class="summary">
      <p>
        Round {view.round} / 4 · {view.round_reveals} / {names.length} revealed
      </p>
      <p>
        {view.revealed.gold} / {view.distribution.cards.gold} gold · {view
          .revealed.traps} / {view.distribution.cards.traps} traps · {view
          .revealed.empty} empty
      </p>
      <p>
        Roles: {view.distribution.adventurers} adventurers · {view.distribution
          .guardians} guardians
      </p>
      <p>Key: {names[view.key]}</p>
    </div>
  </TableSurface>

  <div class="you">
    <p aria-label="Your role">
      You are {view.role === 'adventurer' ? 'an adventurer' : 'a guardian'}
    </p>
    <p aria-label="Your card counts">
      Hidden: {view.hand_counts.gold} gold · {view.hand_counts.traps} traps · {view
        .hand_counts.empty} empty
    </p>
  </div>

  {#if !finished && view.phase === 'inspect'}
    <Notice>
      {view.ready[you]
        ? 'Waiting for other players'
        : 'Check your card counts, then ready'}
    </Notice>
  {/if}
  {#if !finished && ready}
    <Actions><Button onclick={() => move(ready)}>Ready</Button></Actions>
  {/if}

  {#each names as name, player (player)}
    <section aria-label={`${name}'s cards`}>
      <PlayerBar
        {name}
        colorIndex={player}
        you={player === you}
        active={active.includes(player)}
      >
        {#if player === view.key}<span>Key</span>{/if}
        {#if view.phase === 'inspect' && view.ready[player]}<span>Ready</span
          >{/if}
      </PlayerBar>
      <CardRow label={`${name}'s row`}>
        {#each view.cards[player] as card, index (index)}
          {@const action = reveal(player, index)}
          <TempelCard
            {card}
            label={`${name} card ${index + 1}: ${card ?? 'hidden'}`}
            disabled={!action}
            onclick={() => action && move(action)}
          />
        {/each}
      </CardRow>
    </section>
  {/each}
</Board>

<style>
  .summary {
    display: grid;
    justify-items: center;
    gap: var(--arena-space-1);
    font-weight: 700;
    text-align: center;
  }
  .you {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--arena-space-1) var(--arena-space-3);
    font-weight: 700;
  }
</style>
