<script lang="ts">
  import { t } from '../lib/i18n';
  import { suits, type BoardProps, type Schemas } from '../lib/types';
  import CardBack from './cards/CardBack.svelte';
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

  <PlayerBar name={names[you]} colorIndex={you} you active={yourTurn} />
  <CardRow label={$t('crazyEights.yourHand')}>
    {#each view.hand as card, index}
      {@const legal = legalPlay(index)}
      <StandardPlayingCard
        {card}
        selected={wild === index}
        disabled={!legal}
        muted={yourTurn && !legal}
        onclick={() => play(index)}
      />
    {/each}
  </CardRow>

  {#if choices.length}
    <Actions label={$t('crazyEights.chooseSuit')}>
      {#each choices as choice}
        {#if choice.game === 'crazy_eights' && choice.action.type === 'play' && choice.action.suit}
          <Button
            variant="quiet"
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
    </Actions>
  {/if}
  <Actions>
    {#each actions as action}
      {#if action.game === 'crazy_eights' && action.action.type === 'draw'}
        <Button variant="secondary" onclick={() => move(action)}
          >{view.pending_draw
            ? $t('crazyEights.drawTwo')
            : view.stock === 0
              ? $t('crazyEights.drawOrPass')
              : $t('crazyEights.draw')}</Button
        >
      {/if}
    {/each}
  </Actions>
</Board>

<style>
  p {
    font-weight: 700;
  }
</style>
