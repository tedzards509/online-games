<script lang="ts">
  import { t } from '../lib/i18n';
  import type { Action, BoardProps, Schemas } from '../lib/types';
  import Button from './ui/Button.svelte';
  import Board from './table/Board.svelte';
  import CardRow from './table/CardRow.svelte';
  import Pile from './table/Pile.svelte';
  import PlayerBar from './table/PlayerBar.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
  import TableSurface from './table/TableSurface.svelte';
  import TempelCard from './tempel/TempelCard.svelte';

  type View = Schemas['TempelView'];
  type Card = Schemas['TempelCard'];

  let { view, actions, names, move, you, active, finished }: BoardProps<View> =
    $props();
  const revealSeconds = 5;
  /** Counts fields paired with the card each one counts. */
  const kinds = [
    ['gold', 'gold'],
    ['traps', 'trap'],
    ['empty', 'empty'],
  ] as const;

  const available = $derived(
    actions.filter((a) => a.game === 'tempel_des_schreckens'),
  );
  const ready = $derived(available.find((a) => a.action.type === 'ready'));
  const yourTurn = $derived(!finished && active.includes(you));
  const opponents = $derived(
    names.map((_, player) => player).filter((player) => player !== you),
  );
  function reveal(player: number, index: number): Action | undefined {
    return available.find(
      (a) =>
        a.action.type === 'reveal' &&
        a.action.player === player &&
        a.action.index === index,
    );
  }

  /** Cards opened in the round that `before` belongs to, if `after` directly
   * follows its last reveal. The final card never appears in a view: its kind
   * is the change in revealed counts and its owner the new key holder. */
  function finishedRound(before: View, after: View): Card[][] | null {
    if (
      after.round !== before.round + 1 ||
      before.round_reveals !== names.length - 1
    )
      return null;
    const last = kinds.find(
      ([count]) => after.revealed[count] > before.revealed[count],
    );
    if (!last) return null;
    const opened = before.cards.map((row) =>
      row.filter((card): card is Card => card !== null),
    );
    opened[after.key].push(last[1]);
    return opened;
  }

  let roundEnd = $state<{ round: number; cards: Card[][] } | null>(null);
  let seconds = $state(0);
  let previous: View | undefined;
  $effect(() => {
    const cards = previous && finishedRound(previous, view);
    if (cards) {
      roundEnd = { round: previous!.round, cards };
      seconds = revealSeconds;
    }
    previous = view;
  });
  $effect(() => {
    if (!roundEnd) return;
    const timer = setInterval(() => {
      seconds -= 1;
      if (seconds <= 0) roundEnd = null;
    }, 1000);
    return () => clearInterval(timer);
  });

  function cardName(card: Card | null): string {
    return $t(`tempel.cards.${card ?? 'hidden'}`);
  }
  const hint = $derived.by(() => {
    if (finished) return undefined;
    if (view.phase === 'inspect')
      return $t(view.ready[you] ? 'tempel.waiting' : 'tempel.inspect');
    return yourTurn ? $t('tempel.choose') : undefined;
  });
</script>

<Board>
  <TableSurface aria-label={$t('tempel.revealedPiles')}>
    {#if roundEnd}
      <div class="summary">
        <p role="status">
          {$t('tempel.roundEnd', {
            values: { round: roundEnd.round, seconds },
          })}
        </p>
        <div class="opened">
          {#each names as name, player (player)}
            <CardRow label={$t('tempel.openedBy', { values: { name } })}>
              <span class="owner">{name}</span>
              {#each roundEnd.cards[player] as card, index (index)}
                <TempelCard
                  {card}
                  label={cardName(card)}
                  width="var(--arena-card-width-small)"
                />
              {/each}
            </CardRow>
          {/each}
        </div>
        <Button
          variant="secondary"
          size="small"
          onclick={() => (roundEnd = null)}>{$t('tempel.skip')}</Button
        >
      </div>
    {:else}
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
    {/if}
  </TableSurface>

  {#each opponents as player (player)}
    {@const name = names[player]}
    <section aria-label={$t('tempel.playerCards', { values: { name } })}>
      <PlayerBar {name} colorIndex={player} active={active.includes(player)}>
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
              values: { name, index: index + 1, card: cardName(card) },
            })}
            disabled={!action}
            onclick={() => action && move(action)}
          />
        {/each}
      </CardRow>
    </section>
  {/each}

  <PlayerDock name={names[you]} colorIndex={you} attention={yourTurn} {hint}>
    {#snippet details()}
      <span aria-label={$t('tempel.yourRole')}
        >{$t(`tempel.role.${view.role}`)}</span
      >
      {#if you === view.key}<span>{$t('tempel.key')}</span>{/if}
    {/snippet}
    {#snippet hand()}
      <CardRow label={$t('tempel.yourCounts')}>
        {#each kinds as [count, card] (card)}
          <Pile caption={`×${view.hand_counts[count]}`}>
            <TempelCard
              {card}
              label={$t('tempel.count', {
                values: {
                  card: cardName(card),
                  count: view.hand_counts[count],
                },
              })}
              muted={view.hand_counts[count] === 0}
            />
          </Pile>
        {/each}
      </CardRow>
    {/snippet}
    {#snippet controls()}
      {#if ready}
        <Button onclick={() => move(ready)}>{$t('tempel.ready')}</Button>
      {/if}
    {/snippet}
  </PlayerDock>
</Board>

<style>
  section {
    min-width: 0;
  }
  .opened {
    display: grid;
    gap: var(--arena-space-1);
  }
  .owner {
    align-self: center;
    min-width: 5rem;
    text-align: end;
  }
  .summary {
    display: grid;
    justify-items: center;
    gap: var(--arena-space-1);
    font-weight: 700;
    text-align: center;
  }
</style>
