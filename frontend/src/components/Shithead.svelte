<script lang="ts">
  import { t } from '../lib/i18n';
  import { byRank, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import CardSlot from './cards/CardSlot.svelte';
  import StandardPlayingCard from './cards/StandardPlayingCard.svelte';
  import Board from './table/Board.svelte';
  import Hand from './table/Hand.svelte';
  import Pile from './table/Pile.svelte';
  import CardRow from './table/CardRow.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
  import PlayerList from './table/PlayerList.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import Button from './ui/Button.svelte';

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
  const own = $derived(view.players[you]);
  /** The zone you play from: hand, then face-up cards, then face-down cards. */
  const source = $derived(view.hand.length ? view.hand : own.up);
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
  const pickup = $derived(
    actions.find((a) => a.game === 'shithead' && a.action.type === 'pickup'),
  );
  const hint = $derived(
    view.setup
      ? actions.length
        ? $t('shithead.chooseFaceUp')
        : $t('shithead.waiting')
      : !yourTurn
        ? undefined
        : source.length
          ? $t('shithead.playOrPickUp')
          : $t('shithead.flip'),
  );
  $effect(() => {
    view;
    actions;
    selected = [];
  });
  function toggle(index: number): void {
    if (selected.includes(index)) {
      selected = selected.filter((i) => i !== index);
      return;
    }
    // Plays share one rank, so a card of another rank starts a new selection.
    const other =
      selected.length && source[selected[0]].rank !== source[index].rank;
    selected = !view.setup && other ? [index] : [...selected, index];
  }
  function usable(index: number): boolean {
    return actions.some(
      (a) =>
        a.game === 'shithead' &&
        (a.action.type === 'play' || a.action.type === 'select_face_up') &&
        a.action.indices.includes(index),
    );
  }
  function blind(index: number) {
    return actions.find(
      (a) =>
        a.game === 'shithead' &&
        a.action.type === 'blind' &&
        a.action.index === index,
    );
  }
</script>

{#snippet table(player: number)}
  {#each view.players[player].up as card}<StandardPlayingCard
      {card}
      width="var(--arena-card-width-small)"
    />{/each}
{/snippet}

<Board>
  <PlayerList
    players={names.map((_, player) => player).filter((p) => p !== you)}
    {names}
    {you}
    {active}
  >
    {#snippet details(player)}
      {$t('shithead.playerCards', {
        values: {
          count: view.players[player].count,
          down: view.players[player].down,
        },
      })}
      {@render table(player)}
    {/snippet}
  </PlayerList>

  <TableSurface>
    <Pile caption={$t('shithead.stock', { values: { count: view.stock } })}
      ><CardBack /></Pile
    >
    <Pile caption={$t('shithead.pile', { values: { count: view.pile } })}>
      {#if view.recent.length}
        <div class="fan">
          {#each view.recent as card, index}
            <StandardPlayingCard
              {card}
              highlighted={index === view.recent.length - 1}
            />
          {/each}
        </div>
      {:else}<CardSlot label={$t('shithead.emptyPile')} />{/if}
    </Pile>
    {#if view.failed_blind}
      <Pile
        caption={$t('shithead.failedBlind', {
          values: { name: names[view.failed_blind.player] },
        })}
      >
        <StandardPlayingCard card={view.failed_blind.card} />
      </Pile>
    {/if}
  </TableSurface>

  <PlayerDock name={names[you]} colorIndex={you} attention={yourTurn} {hint}>
    {#snippet details()}
      {#if source.length && own.down}{$t('shithead.faceDownCount', {
          values: { count: own.down },
        })}{/if}
    {/snippet}
    {#snippet hand()}
      {#if view.hand.length && own.up.length}
        <CardRow label={$t('shithead.yourFaceUp')}>{@render table(you)}</CardRow
        >
      {/if}
      {#if source.length}
        <Hand
          label={view.hand.length
            ? $t('shithead.yourHand')
            : $t('shithead.yourFaceUp')}
          items={source}
          order={byRank}
        >
          {#snippet card(card, index)}
            <StandardPlayingCard
              {card}
              selected={selected.includes(index)}
              disabled={!usable(index)}
              muted={yourTurn && !usable(index)}
              onclick={() => toggle(index)}
            />
          {/snippet}
        </Hand>
      {:else if own.down}
        <Hand
          label={$t('shithead.yourFaceDown')}
          items={Array.from({ length: own.down }, (_, index) => index)}
        >
          {#snippet card(index)}
            {@const action = blind(index)}
            <CardBack
              disabled={!action}
              label={$t('shithead.faceDownCard', {
                values: { number: index + 1 },
              })}
              onclick={() => action && move(action)}
            />
          {/snippet}
        </Hand>
      {/if}
    {/snippet}
    {#snippet controls()}
      {#if pickup}<Button variant="secondary" onclick={() => move(pickup)}
          >{$t('shithead.pickUp')}</Button
        >{/if}
      {#if source.length}<Button
          disabled={!playable}
          onclick={() => playable && move(playable)}
          >{view.setup ? $t('shithead.setFaceUp') : $t('shithead.play')}</Button
        >{/if}
    {/snippet}
  </PlayerDock>
</Board>

<style>
  /* Older pile cards peek out to the left; the top card is fully visible. */
  .fan {
    display: flex;
  }
  .fan > :global(* + *) {
    margin-left: calc(var(--arena-card-width) * -0.7);
  }
</style>
