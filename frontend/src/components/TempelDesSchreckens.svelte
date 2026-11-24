<script lang="ts">
  import { t } from '../lib/i18n';
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
  <TableSurface aria-label={$t('tempel.revealedPiles')}>
    <div class="summary">
      <p>
        {$t('tempel.round', {
          values: {
            round: view.round,
            reveals: view.round_reveals,
            players: names.length,
          },
        })}
      </p>
      <p>
        {$t('tempel.revealed', {
          values: {
            gold: view.revealed.gold,
            goldTotal: view.distribution.cards.gold,
            traps: view.revealed.traps,
            trapsTotal: view.distribution.cards.traps,
            empty: view.revealed.empty,
          },
        })}
      </p>
      <p>
        {$t('tempel.roles', {
          values: {
            adventurers: view.distribution.adventurers,
            guardians: view.distribution.guardians,
          },
        })}
      </p>
      <p>{$t('tempel.keyHolder', { values: { name: names[view.key] } })}</p>
    </div>
  </TableSurface>

  <div class="you">
    <p aria-label={$t('tempel.yourRole')}>{$t(`tempel.role.${view.role}`)}</p>
    <p aria-label={$t('tempel.yourCounts')}>
      {$t('tempel.hidden', { values: { ...view.hand_counts } })}
    </p>
  </div>

  {#if !finished && view.phase === 'inspect'}
    <Notice>
      {$t(view.ready[you] ? 'tempel.waiting' : 'tempel.inspect')}
    </Notice>
  {/if}
  {#if !finished && ready}
    <Actions>
      <Button onclick={() => move(ready)}>{$t('tempel.ready')}</Button>
    </Actions>
  {/if}

  {#each names as name, player (player)}
    <section aria-label={$t('tempel.playerCards', { values: { name } })}>
      <PlayerBar
        {name}
        colorIndex={player}
        you={player === you}
        active={active.includes(player)}
      >
        {#if player === view.key}<span>{$t('tempel.key')}</span>{/if}
        {#if view.phase === 'inspect' && view.ready[player]}<span
            >{$t('tempel.ready')}</span
          >{/if}
      </PlayerBar>
      <CardRow label={$t('tempel.playerRow', { values: { name } })}>
        {#each view.cards[player] as card, index (index)}
          {@const action = reveal(player, index)}
          <TempelCard
            {card}
            label={$t('tempel.cardLabel', {
              values: {
                name,
                index: index + 1,
                card: $t(`tempel.cards.${card ?? 'hidden'}`),
              },
            })}
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
