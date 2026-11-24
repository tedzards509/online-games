<script lang="ts">
  import { t } from '../lib/i18n';
  import { byRank, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import CardSlot from './cards/CardSlot.svelte';
  import StandardPlayingCard from './cards/StandardPlayingCard.svelte';
  import CardTable from './table/CardTable.svelte';
  import Hand from './table/Hand.svelte';
  import OpponentSeat from './table/OpponentSeat.svelte';
  import Pile from './table/Pile.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
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
  const opponents = $derived(
    Array.from(
      { length: names.length - 1 },
      (_, offset) => (you + offset + 1) % names.length,
    ),
  );
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
        ? $t('common.playerTurn', { values: { name: names[active[0]] } })
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

{#snippet tableCards(player: number)}
  {@const seat = view.players[player]}
  <div
    class="table-cards"
    role="group"
    aria-label={$t('shithead.playerCards', {
      values: { count: seat.count, down: seat.down },
    })}
  >
    {#each { length: Math.max(seat.up.length, seat.down) }, slot}
      <div class="stack">
        {#if slot < seat.down}<CardBack
            width="var(--shithead-table-card)"
          />{/if}
        {#if slot < seat.up.length}<StandardPlayingCard
            card={seat.up[slot]}
            width="var(--shithead-table-card)"
          />{/if}
      </div>
    {/each}
  </div>
{/snippet}

<CardTable {opponents}>
  {#snippet opponent(player)}
    <OpponentSeat
      name={names[player]}
      colorIndex={player}
      cards={view.players[player].count}
      active={active.includes(player)}
    >
      {#snippet markers()}
        {@render tableCards(player)}
      {/snippet}
    </OpponentSeat>
  {/snippet}
  {#snippet center()}
    <Pile caption={$t('shithead.stock', { values: { count: view.stock } })}>
      {#if view.stock}<CardBack />{:else}<CardSlot
          label={$t('shithead.emptyStock')}
        />{/if}
    </Pile>
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
  {/snippet}
  {#snippet dock()}
    <PlayerDock name={names[you]} colorIndex={you} attention={yourTurn} {hint}>
      {#snippet details()}
        {#if !view.hand.length && own.up.length && own.down}{$t(
            'shithead.faceDownCount',
            {
              values: { count: own.down },
            },
          )}{/if}
      {/snippet}
      {#snippet hand()}
        <!-- Your table cards sit beside the hand while you still hold one. -->
        <div class="own">
          {#if view.hand.length}{@render tableCards(you)}{/if}
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
        </div>
      {/snippet}
      {#snippet controls()}
        {#if pickup}<Button variant="secondary" onclick={() => move(pickup)}
            >{$t('shithead.pickUp')}</Button
          >{/if}
        {#if source.length}<Button
            disabled={!playable}
            onclick={() => playable && move(playable)}
            >{view.setup
              ? $t('shithead.setFaceUp')
              : $t('shithead.play')}</Button
          >{/if}
      {/snippet}
    </PlayerDock>
  {/snippet}
</CardTable>

<style>
  /* Face-up cards lie on the face-down ones, offset so both stay visible. */
  .table-cards {
    /* Three stacks fit a seat two card widths wide. */
    --shithead-table-card: calc(var(--arena-card-width) * 0.66);
    display: flex;
    justify-content: center;
    gap: calc(var(--arena-card-width) * 0.05);
  }
  .own {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--arena-space-1) var(--arena-space-3);
    min-width: 0;
  }
  .own > :global(:last-child) {
    min-width: 0;
  }
  .stack {
    display: grid;
  }
  .stack > :global(*) {
    grid-area: 1 / 1;
  }
  .stack > :global(* + *) {
    margin-top: calc(var(--shithead-table-card) * 0.45);
  }
  /* Older pile cards peek out to the left; the top card is fully visible. */
  .fan {
    display: flex;
  }
  .fan > :global(* + *) {
    margin-left: calc(var(--arena-card-width) * -0.7);
  }
</style>
