<script lang="ts">
  import { t } from '../lib/i18n';
  import { byRank, suits, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
  import StandardPlayingCard from './cards/StandardPlayingCard.svelte';
  import Board from './table/Board.svelte';
  import Hand from './table/Hand.svelte';
  import Pile from './table/Pile.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
  import PlayerList from './table/PlayerList.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import Button from './ui/Button.svelte';
  import Notice from './ui/Notice.svelte';

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

<Board>
  <PlayerList
    players={names.map((_, player) => player).filter((p) => p !== you)}
    {names}
    {you}
    {active}
  >
    {#snippet details(player)}{$t('crazyEights.cards', {
        values: { count: view.counts[player] },
      })}{/snippet}
  </PlayerList>

  <TableSurface>
    <Pile caption={$t('crazyEights.stock', { values: { count: view.stock } })}
      ><CardBack /></Pile
    >
    <Pile
      caption={$t('crazyEights.suit', { values: { suit: suits[view.suit] } })}
    >
      <StandardPlayingCard card={view.top} highlighted />
    </Pile>
    {#if view.reversed}<p>{$t('crazyEights.reversed')}</p>{/if}
  </TableSurface>

  {#if view.pending_draw}<Notice
      >{view.pending_draw === 'ace'
        ? $t('crazyEights.aceOrDrawTwo')
        : $t('crazyEights.drawTwo')}</Notice
    >{/if}

  <PlayerDock
    name={names[you]}
    colorIndex={you}
    attention={yourTurn}
    hint={wild !== null
      ? $t('crazyEights.chooseSuit')
      : yourTurn
        ? $t('crazyEights.playOrDraw')
        : undefined}
  >
    {#snippet hand()}
      <Hand label={$t('crazyEights.yourHand')} items={view.hand} order={byRank}>
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
</Board>

<style>
  p {
    font-weight: 700;
  }
  .suits {
    display: flex;
    gap: var(--arena-space-1);
  }
</style>
