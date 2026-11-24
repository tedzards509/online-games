<script lang="ts">
  import type { Action, BoardProps, Schemas, Snapshot } from '../lib/types';
  import { t } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import CrewCard from './crew/CrewCard.svelte';
  import CrewPlayer from './crew/CrewPlayer.svelte';
  import CommunicationSlot from './crew/CommunicationSlot.svelte';
  import TrickPile from './crew/TrickPile.svelte';
  import CardTable from './table/CardTable.svelte';
  import Hand from './table/Hand.svelte';
  import PlayerDock from './table/PlayerDock.svelte';
  import { cardKey, crewColors, trickLeader } from './crew/cards';

  let {
    view,
    actions,
    players,
    you,
    move,
    finished,
    disabled,
    roomInfo,
    won,
  }: BoardProps<Schemas['CrewView']> & {
    /** Public member identities by player index. */
    players: Schemas['MemberView'][];
    /** Gates commands only; cards stay selectable while pending or offline. */
    disabled: boolean;
    roomInfo?: Pick<Snapshot, 'code' | 'host'>;
    won: boolean;
  } = $props();

  let selected = $state<string | null>(null);
  const selectedIndex = $derived(
    view.hand.findIndex((card) => cardKey(card) === selected),
  );
  const crewActions = $derived(
    actions.filter((action) => action.game === 'the_crew'),
  );
  const opponents = $derived(
    Array.from(
      { length: players.length - 1 },
      (_, offset) => (you + offset + 1) % players.length,
    ),
  );
  const currentCards = $derived(
    view.trick.length ? view.trick : (view.last_trick?.cards ?? []),
  );
  const winningPlayer = $derived(
    view.trick.length
      ? trickLeader(view.trick)
      : (view.last_trick?.winner ?? null),
  );
  const communicationAvailable = $derived(
    crewActions.some((action) => action.action.type === 'communicate'),
  );
  const yourTurn = $derived(
    !finished && !view.awaiting_result && view.next_player === you,
  );
  const declarations = ['highest', 'only', 'lowest'] as const;

  function cardName(card: Schemas['CrewCard']): string {
    return $t('crew.card', {
      values: {
        color: $t(`crew.colors.${card.color}`),
        number: String(card.number),
      },
    });
  }
  function play(index: number): Action | undefined {
    return crewActions.find(
      (action) =>
        action.action.type === 'play' && action.action.index === index,
    );
  }
  function communicate(
    declaration: Schemas['CrewDeclaration'],
  ): Action | undefined {
    return crewActions.find(
      (action) =>
        action.action.type === 'communicate' &&
        action.action.index === selectedIndex &&
        action.action.declaration === declaration,
    );
  }
  function submit(action: Action | undefined): void {
    // Retain selection until the authoritative hand changes, including on errors.
    if (action && !disabled && !finished) move(action);
  }
</script>

{#snippet trickCards(
  cards: Schemas['CrewPlay'][],
  winner: number | null,
  wrap = false,
)}
  <div class="trick-cards" class:wrap>
    {#each cards as played (played.player)}
      <figure class:winning={played.player === winner}>
        <CrewCard
          card={played.card}
          highlighted={played.player === winner}
          label={played.player === winner
            ? $t('crew.winningCard', {
                values: { card: cardName(played.card) },
              })
            : cardName(played.card)}
        />
        <figcaption title={players[played.player].nickname}>
          {players[played.player].nickname}
        </figcaption>
      </figure>
    {/each}
  </div>
{/snippet}

<CardTable {opponents}>
  {#snippet opponent(player)}
    <CrewPlayer
      {player}
      member={players[player]}
      {view}
      {roomInfo}
      {finished}
    />
  {/snippet}
  {#snippet center()}
    <section
      class="trick"
      aria-label={$t(
        view.trick.length || !view.last_trick
          ? 'crew.currentTrick'
          : 'crew.lastTrick',
      )}
    >
      <p class="trick-title">
        {$t(
          view.trick.length || !view.last_trick
            ? 'crew.currentTrick'
            : 'crew.lastTrick',
        )}
      </p>
      {@render trickCards(currentCards, winningPlayer)}
      {#if !view.trick.length && view.last_trick}<p class="winner">
          {$t('crew.trickWinner', {
            values: { name: players[view.last_trick.winner].nickname },
          })}
        </p>{/if}
    </section>
  {/snippet}
  {#snippet dock()}
    <PlayerDock
      name={players[you].nickname}
      colorIndex={you}
      attention={yourTurn}
      hint={finished
        ? $t(won ? 'common.won' : 'common.lost')
        : view.awaiting_result
          ? $t('crew.awaitingResult')
          : yourTurn
            ? $t('common.yourTurn')
            : $t(view.trick.length ? 'crew.playsNext' : 'crew.leads', {
                values: { name: players[view.next_player].nickname },
              })}
    >
      {#snippet hand()}
        <div class="own-cards">
          <div class="own-communication">
            <CommunicationSlot
              communication={view.communications[you]}
              played={view.communication_played[you]}
              name={players[you].nickname}
            />
            {#if view.commander === you}<span
                class="commander"
                style:background={crewColors.black}
                role="img"
                aria-label={$t('crew.commander')}
                title={$t('crew.commander')}
              ></span>{/if}
          </div>
          <div class="own-hand">
            <Hand label={$t('crew.yourHand')} items={view.hand}>
              {#snippet card(card, index)}
                <CrewCard
                  {card}
                  muted={yourTurn && !play(index)}
                  label={$t('crew.selectCard', {
                    values: { card: cardName(card) },
                  })}
                  selected={selectedIndex === index}
                  onclick={() =>
                    (selected = selectedIndex === index ? null : cardKey(card))}
                />
              {/snippet}
            </Hand>
          </div>
          <div class="own-tricks">
            <TrickPile
              count={view.tricks_won[you]}
              name={players[you].nickname}
            />
          </div>
        </div>
      {/snippet}
      {#snippet controls()}
        {#if view.last_trick && view.trick.length}<Button
            variant="quiet"
            size="small"
            square
            popovertarget="crew-last-trick"
            aria-label={$t('crew.lastTrick')}
            title={$t('crew.lastTrick')}
            ><svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
              ><circle cx="12" cy="12" r="9" /><path d="M12 6v6l4 2" /></svg
            ></Button
          >{/if}
        {#if communicationAvailable && !finished}<Button
            variant="secondary"
            size="small"
            popovertarget="crew-communicate"
            aria-label={$t('crew.communicateCard')}
            disabled={disabled ||
              !declarations.some((declaration) => communicate(declaration))}
            >{$t('crew.communicate')}</Button
          >{/if}
        <Button
          size="small"
          disabled={disabled || finished || !play(selectedIndex)}
          onclick={() => submit(play(selectedIndex))}
          >{$t('crew.playCard')}</Button
        >
      {/snippet}
    </PlayerDock>
  {/snippet}
</CardTable>

{#if communicationAvailable && !finished}
  <section
    class="table-popover"
    popover
    id="crew-communicate"
    aria-label={$t('crew.communicateCard')}
  >
    <h2>{$t('crew.communicateCard')}</h2>
    <p>
      {selectedIndex >= 0
        ? cardName(view.hand[selectedIndex])
        : $t('crew.selectPrompt')}
    </p>
    <div class="declarations">
      {#each declarations as declaration}<Button
          variant="secondary"
          disabled={disabled || !communicate(declaration)}
          onclick={() => submit(communicate(declaration))}
          >{$t(`crew.declarations.${declaration}`)}</Button
        >{/each}
    </div>
    <Button
      variant="quiet"
      size="small"
      popovertarget="crew-communicate"
      popovertargetaction="hide">{$t('common.close')}</Button
    >
  </section>
{/if}
{#if view.last_trick && view.trick.length}
  <section
    class="table-popover"
    popover
    id="crew-last-trick"
    aria-label={$t('crew.lastTrick')}
  >
    <h2>{$t('crew.lastTrick')}</h2>
    <p>
      {$t('crew.trickWinner', {
        values: { name: players[view.last_trick.winner].nickname },
      })}
    </p>
    {@render trickCards(view.last_trick.cards, view.last_trick.winner, true)}
    <Button
      variant="quiet"
      size="small"
      popovertarget="crew-last-trick"
      popovertargetaction="hide">{$t('common.close')}</Button
    >
  </section>
{/if}

<style>
  .trick {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
  }
  .trick-title,
  .winner {
    margin: 0.5rem 0;
    font-size: 0.8rem;
  }
  .trick-cards {
    display: flex;
    justify-content: safe center;
    gap: 0.75rem;
    padding: 0.5rem;
    max-width: 100%;
    overflow-x: auto;
    flex-shrink: 0;
  }
  .trick-cards.wrap {
    flex-wrap: wrap;
    overflow: visible;
    justify-content: center;
  }
  figure {
    margin: 0;
    width: var(--arena-card-width);
    flex: 0 0 var(--arena-card-width);
  }
  figcaption {
    font-size: 0.75rem;
    margin-top: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .own-cards {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 1rem;
  }
  .own-hand {
    min-width: 0;
  }
  .own-communication {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .commander {
    width: calc(var(--arena-card-width) * 0.75);
    aspect-ratio: 1;
    flex-shrink: 0;
    border-radius: 50%;
    border: var(--arena-line);
  }
  .declarations {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
  }
  .table-popover {
    width: min(28rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    padding: 1.5rem;
    overflow: auto;
    border: var(--arena-line);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    box-shadow: var(--arena-overlay);
  }
  .table-popover:popover-open {
    display: grid;
    justify-items: start;
    gap: 1rem;
  }
  @media (max-width: 40rem) {
    .own-cards,
    .own-communication {
      gap: 0.5rem;
    }
    .trick-cards {
      gap: 0.25rem;
    }
  }
  @media (max-height: 500px) and (min-width: 40.01rem) {
    .trick-title,
    .winner {
      margin: 0.125rem 0;
    }
  }
  @media (max-width: 40rem) and (orientation: portrait) {
    .own-cards {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      grid-template-areas: 'communication tricks' 'hand hand';
    }
    .trick-title,
    .winner {
      margin: 0.25rem 0;
    }
    /* Markers shrink so the hand and trick keep their full card size. */
    .own-communication,
    .own-tricks {
      --arena-card-width: var(--arena-card-width-small);
    }
    .own-communication {
      grid-area: communication;
      justify-content: start;
    }
    .own-tricks {
      grid-area: tricks;
      justify-self: end;
    }
    .own-hand {
      grid-area: hand;
    }
  }
</style>
