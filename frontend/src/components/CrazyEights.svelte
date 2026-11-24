<script lang="ts">
  import { t } from '../lib/i18n';
  import { byRank, suits, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import StandardPlayingCard from './cards/StandardPlayingCard.svelte';
  import CardSlot from './cards/CardSlot.svelte';
  import CardTable from './table/CardTable.svelte';
  import Hand from './table/Hand.svelte';
  import Pile from './table/Pile.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
  import OpponentSeat from './table/OpponentSeat.svelte';
  import Button from './ui/Button.svelte';

  let {
    view,
    actions,
    names,
    you,
    active,
    move,
  }: BoardProps<Schemas['CrazyEightsView']> = $props();
  let wild = $state<number | null>(null);
  const yourTurn = $derived(active.includes(you));
  const opponents = $derived(
    Array.from(
      { length: names.length - 1 },
      (_, offset) => (you + offset + 1) % names.length,
    ),
  );
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
  function legalPlay(index: number) {
    return actions.find(
      (a) =>
        a.game === 'crazy_eights' &&
        a.action.type === 'play' &&
        a.action.index === index,
    );
  }
  function play(index: number): void {
    const legal = legalPlay(index);
    if (!legal || legal.game !== 'crazy_eights') return;
    if (legal.action.type === 'play' && legal.action.suit) wild = index;
    else move(legal);
  }
</script>

<CardTable {opponents}>
  {#snippet opponent(player)}
    <OpponentSeat
      name={names[player]}
      colorIndex={player}
      cards={view.counts[player]}
      active={active.includes(player)}
    >
      {#snippet markers()}<p>
          {$t('crazyEights.cards', { values: { count: view.counts[player] } })}
        </p>{/snippet}
    </OpponentSeat>
  {/snippet}
  {#snippet center()}
    <Pile caption={$t('crazyEights.stock', { values: { count: view.stock } })}>
      {#if view.stock}<CardBack />{:else}<CardSlot
          label={$t('crazyEights.emptyStock')}
        />{/if}
    </Pile>
    <Pile
      caption={$t('crazyEights.suit', { values: { suit: suits[view.suit] } })}
    >
      <StandardPlayingCard card={view.top} highlighted />
    </Pile>
    {#if view.reversed}<p>{$t('crazyEights.reversed')}</p>{/if}
    {#if view.pending_draw}<p>
        {view.pending_draw === 'ace'
          ? $t('crazyEights.aceOrDrawTwo')
          : $t('crazyEights.drawTwo')}
      </p>{/if}
  {/snippet}
  {#snippet dock()}
    <PlayerDock
      name={names[you]}
      colorIndex={you}
      attention={yourTurn}
      hint={wild !== null
        ? $t('crazyEights.chooseSuit')
        : yourTurn
          ? $t('crazyEights.playOrDraw')
          : active.length
            ? $t('common.playerTurn', { values: { name: names[active[0]] } })
            : undefined}
    >
      {#snippet hand()}
        <Hand
          label={$t('crazyEights.yourHand')}
          items={view.hand}
          order={byRank}
        >
          {#snippet card(card, index)}
            {@const legal = legalPlay(index)}
            <StandardPlayingCard
              {card}
              selected={wild === index}
              disabled={!legal}
              muted={yourTurn && !legal}
              onclick={() => play(index)}
            />
          {/snippet}
        </Hand>
      {/snippet}
      {#snippet controls()}
        {#if choices.length}
          <div
            class="suits"
            role="group"
            aria-label={$t('crazyEights.chooseSuit')}
          >
            {#each choices as choice}
              {#if choice.game === 'crazy_eights' && choice.action.type === 'play' && choice.action.suit}
                <Button
                  variant="quiet"
                  square
                  aria-label={$t('crazyEights.chooseSuitOf', {
                    values: { suit: $t(`suits.${choice.action.suit}`) },
                  })}
                  onclick={() => {
                    move(choice);
                    wild = null;
                  }}>{suits[choice.action.suit]}</Button
                >
              {/if}
            {/each}
          </div>
        {/if}
        {#each actions as action}
          {#if action.game === 'crazy_eights' && action.action.type === 'draw'}
            <Button onclick={() => move(action)}
              >{view.pending_draw
                ? $t('crazyEights.drawTwo')
                : view.stock === 0
                  ? $t('crazyEights.drawOrPass')
                  : $t('crazyEights.draw')}</Button
            >
          {/if}
        {/each}
      {/snippet}
    </PlayerDock>
  {/snippet}
</CardTable>

<style>
  p {
    margin: 0;
    font-weight: 700;
  }
  .suits {
    display: flex;
    gap: var(--arena-space-1);
  }
</style>
