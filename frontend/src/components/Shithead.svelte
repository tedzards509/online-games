<script lang="ts">
  import { cardLabel, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import CardSlot from './cards/CardSlot.svelte';
  import StandardPlayingCard from './cards/StandardPlayingCard.svelte';
  import Actions from './table/Actions.svelte';
  import CardRow from './table/CardRow.svelte';
  import Pile from './table/Pile.svelte';
  import Board from './table/Board.svelte';
  import PlayerBar from './table/PlayerBar.svelte';
  import PlayerList from './table/PlayerList.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import Button from './ui/Button.svelte';
  import Notice from './ui/Notice.svelte';

  let {
    view,
    actions,
    you,
    names,
    active,
    move,
  }: BoardProps<Schemas['ShitheadView']> = $props();
  let selected = $state<number[]>([]);
  const yourTurn = $derived(active.includes(you));
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
  function usable(index: number): boolean {
    return actions.some(
      (a) =>
        a.game === 'shithead' &&
        (a.action.type === 'play' || a.action.type === 'select_face_up') &&
        a.action.indices.includes(index),
    );
  }
</script>

<Board>
  <PlayerList
    players={names.map((_, player) => player).filter((p) => p !== you)}
    {names}
    {you}
    {active}
  >
    {#snippet details(player)}
      {view.players[player].count} in hand · {view.players[player].down} down
      {#each view.players[player].up as card}<StandardPlayingCard
          {card}
          label={cardLabel(card)}
          width="var(--arena-card-width-small)"
        />{/each}
    {/snippet}
  </PlayerList>

  <TableSurface>
    <Pile caption={`${view.stock} in stock`}><CardBack /></Pile>
    <Pile caption={`${view.pile} in pile`}>
      {#if view.top}<StandardPlayingCard
          card={view.top}
          label={cardLabel(view.top)}
          highlighted
        />{:else}<CardSlot label="Empty pile" />{/if}
    </Pile>
    {#if view.top?.rank === 3 && view.effective_top}
      <Pile caption="3 is transparent">
        <StandardPlayingCard
          card={view.effective_top}
          label={cardLabel(view.effective_top)}
        />
      </Pile>
    {/if}
  </TableSurface>

  {#if view.setup}<Notice
      >{actions.length
        ? 'Choose three face-up cards'
        : 'Waiting for other players'}</Notice
    >{/if}

  <PlayerBar name={names[you]} colorIndex={you} you active={yourTurn}>
    {#if view.hand.length}
      {#each view.players[you].up as card}<StandardPlayingCard
          {card}
          label={cardLabel(card)}
          width="var(--arena-card-width-small)"
        />{/each}
    {/if}
  </PlayerBar>
  <CardRow label={view.hand.length ? 'Your hand' : 'Your face-up cards'}>
    {#each source as card, index}
      <StandardPlayingCard
        {card}
        label={cardLabel(card)}
        selected={selected.includes(index)}
        disabled={!usable(index)}
        muted={yourTurn && !usable(index)}
        onclick={() => toggle(index)}
      />
    {/each}
  </CardRow>
  {#if view.players[you].down}
    <CardRow label="Your face-down cards">
      {#each Array(view.players[you].down) as _, index}
        {@const action = actions.find(
          (a) =>
            a.game === 'shithead' &&
            a.action.type === 'blind' &&
            a.action.index === index,
        )}
        <CardBack
          disabled={!action}
          label={`Face-down card ${index + 1}`}
          onclick={() => action && move(action)}
        />
      {/each}
    </CardRow>
  {/if}

  <Actions>
    {#if source.length}<Button
        disabled={!playable}
        onclick={() => playable && move(playable)}
        >{view.setup ? 'Set face-up cards' : 'Play'}</Button
      >{/if}
    {#each actions as action}
      {#if action.game === 'shithead' && action.action.type === 'pickup'}<Button
          variant="secondary"
          onclick={() => move(action)}>Pick up</Button
        >{/if}
    {/each}
  </Actions>
</Board>

<style>
</style>
