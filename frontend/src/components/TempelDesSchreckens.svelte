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
  const rounds = 4;
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
  let previous: View | undefined;
  $effect(() => {
    const cards = previous && finishedRound(previous, view);
    if (cards) roundEnd = { round: previous!.round, cards };
    previous = view;
  });
  /** The finished round stays visible until the viewer is ready for the next. */
  const summary = $derived(
    roundEnd && view.phase === 'inspect' && !view.ready[you] ? roundEnd : null,
  );

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

{#snippet keyMark(player: number)}
  {#if player === view.key}
    <svg
      class="key"
      viewBox="0 0 24 24"
      role="img"
      aria-label={$t('tempel.keyHolder', { values: { name: names[player] } })}
    >
      <circle cx="8" cy="12" r="4.5" />
      <path d="M12.5 12H21m-3 0v3.5M15 12v2.5" />
    </svg>
  {/if}
{/snippet}

{#snippet pips(filled: number, total: number, label: string)}
  <span class="pips" role="img" aria-label={label}>
    {#each { length: total }, index}<span class:filled={index < filled}
      ></span>{/each}
  </span>
{/snippet}

<Board>
  <TableSurface aria-label={$t('tempel.revealedPiles')}>
    {#if summary}
      <div class="summary">
        <h2>{$t('tempel.lastRound', { values: { round: summary.round } })}</h2>
        <div class="opened">
          {#each names as name, player (player)}
            <CardRow label={$t('tempel.openedBy', { values: { name } })}>
              <span class="owner">{name}</span>
              {#each summary.cards[player] as card, index (index)}
                <TempelCard
                  {card}
                  label={cardName(card)}
                  width="var(--arena-card-width-small)"
                />
              {/each}
            </CardRow>
          {/each}
        </div>
      </div>
    {:else}
      <div class="summary">
        {@render pips(
          view.round,
          rounds,
          $t('tempel.roundOf', { values: { round: view.round, rounds } }),
        )}
        <div class="found">
          {#each kinds as [count, card] (card)}
            {@const total =
              card === 'empty' ? null : view.distribution.cards[count]}
            <Pile
              caption={total === null
                ? `${view.revealed[count]}`
                : `${view.revealed[count]} / ${total}`}
            >
              <TempelCard
                {card}
                label={$t(total === null ? 'tempel.count' : 'tempel.found', {
                  values: {
                    card: cardName(card),
                    count: view.revealed[count],
                    total: total ?? 0,
                  },
                })}
                width="var(--arena-card-width-small)"
              />
            </Pile>
          {/each}
        </div>
        {#if view.phase === 'reveal'}
          {@render pips(
            view.round_reveals,
            names.length,
            $t('tempel.opened', {
              values: { count: view.round_reveals, total: names.length },
            }),
          )}
        {/if}
      </div>
    {/if}
  </TableSurface>

  {#each opponents as player (player)}
    {@const name = names[player]}
    <section aria-label={$t('tempel.playerCards', { values: { name } })}>
      <PlayerBar {name} colorIndex={player} active={active.includes(player)}>
        {@render keyMark(player)}
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
      {@render keyMark(you)}
    {/snippet}
    {#snippet hand()}
      <div class="own">
        <CardRow
          label={$t('tempel.playerRow', { values: { name: names[you] } })}
        >
          {#each view.cards[you] as card, index (index)}
            <TempelCard
              {card}
              label={$t('tempel.cardLabel', {
                values: {
                  name: names[you],
                  index: index + 1,
                  card: cardName(card),
                },
              })}
              width="var(--arena-card-width-small)"
            />
          {/each}
        </CardRow>
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
                width="var(--arena-card-width-small)"
              />
            </Pile>
          {/each}
        </CardRow>
      </div>
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
    gap: var(--arena-space-2);
    font-weight: 700;
    text-align: center;
  }
  .found,
  .own {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    align-items: end;
    gap: var(--arena-space-1) var(--arena-space-3);
  }
  .pips {
    display: flex;
    gap: 0.4rem;
  }
  .pips > span {
    width: 0.75rem;
    aspect-ratio: 1;
    border: 2px solid currentColor;
    border-radius: 50%;
  }
  .pips > .filled {
    background: currentColor;
  }
  .key {
    width: 1.4rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.5;
    stroke-linecap: round;
  }
</style>
