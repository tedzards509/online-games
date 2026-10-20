<script lang="ts">
  import type { Action, Schemas, Snapshot } from '../lib/types';
  import { locale, t } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import CrewCard from './crew/CrewCard.svelte';
  import CrewPlayer from './crew/CrewPlayer.svelte';
  import CommunicationSlot from './crew/CommunicationSlot.svelte';
  import TrickPile from './crew/TrickPile.svelte';
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
  }: {
    view: Schemas['CrewView'];
    actions: Action[];
    players: Schemas['MemberView'][];
    you: number;
    move: (action: Action) => void;
    finished: boolean;
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
  /** Stable visual scatter for public trick cards only. */
  function scatter(played: Schemas['CrewPlay']): string {
    const value =
      played.player * 17 + played.card.number * 7 + played.card.color.length;
    return `translateY(${(value % 7) - 3}px) rotate(${(value % 9) - 4}deg)`;
  }
</script>

{#snippet trickCards(
  cards: Schemas['CrewPlay'][],
  scattered = false,
  winner: number | null = null,
)}
  <div class="trick-cards" class:history-cards={!scattered}>
    {#each cards as played (played.player)}
      <figure
        class:winning={played.player === winner}
        style:transform={scattered ? scatter(played) : undefined}
      >
        <CrewCard
          card={played.card}
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

<div
  class="crew-table"
  style:--crew-black={crewColors.black}
  lang={$locale === 'de-cool' ? 'de' : $locale}
>
  <div
    class="arena"
    data-count={opponents.length}
    style:--opponents={opponents.length}
  >
    <div class="opponents">
      {#each opponents as player (player)}<div class="opponent">
          <CrewPlayer
            {player}
            member={players[player]}
            {view}
            {roomInfo}
            {finished}
          />
        </div>{/each}
    </div>

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
      {@render trickCards(currentCards, true, winningPlayer)}
      {#if !view.trick.length && view.last_trick}<p class="winner">
          {$t('crew.trickWinner', {
            values: { name: players[view.last_trick.winner].nickname },
          })}
        </p>{/if}
    </section>
  </div>
  <div class="own-zone">
    <p class="status" class:your-turn={yourTurn} role="status">
      {#if finished}{$t(won ? 'crew.won' : 'crew.lost')}
      {:else if view.awaiting_result}{$t('crew.awaitingResult')}
      {:else if yourTurn}{$t('crew.yourTurn')}
      {:else}{$t(view.trick.length ? 'crew.playsNext' : 'crew.leads', {
          values: { name: players[view.next_player].nickname },
        })}{/if}
    </p>
    <div class="own-cards">
      <div class="own-communication">
        <CommunicationSlot
          communication={view.communications[you]}
          played={view.communication_played[you]}
          name={players[you].nickname}
        />
        {#if view.commander === you}<span
            class="commander"
            role="img"
            aria-label={$t('crew.commander')}
            title={$t('crew.commander')}
          ></span>{/if}
      </div>
      <section class="hand-scroll" aria-label={$t('crew.yourHand')}>
        <div class="card-row">
          {#each view.hand as card, index (cardKey(card))}
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
          {/each}
        </div>
      </section>
      <div class="own-tricks">
        <TrickPile count={view.tricks_won[you]} name={players[you].nickname} />
      </div>
    </div>
    <div class="controls">
      <Button
        disabled={disabled || finished || !play(selectedIndex)}
        onclick={() => submit(play(selectedIndex))}
        >{$t('crew.playCard')}</Button
      >
      {#if communicationAvailable && !finished}<Button
          variant="secondary"
          popovertarget="crew-communicate"
          aria-label={$t('crew.communicateCard')}
          disabled={disabled ||
            !declarations.some((declaration) => communicate(declaration))}
          >{$t('crew.communicate')}</Button
        >{/if}
      {#if view.last_trick && view.trick.length}<button
          class="history"
          popovertarget="crew-last-trick"
          aria-label={$t('crew.lastTrick')}
          title={$t('crew.lastTrick')}
          ><svg
            viewBox="0 0 24 24"
            width="20"
            height="20"
            fill="none"
            stroke="currentColor"
            stroke-width="1.7"
            aria-hidden="true"
            ><circle cx="12" cy="12" r="9" /><path d="M12 6v6l4 2" /></svg
          ></button
        >{/if}
    </div>
  </div>

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
      <button popovertarget="crew-communicate" popovertargetaction="hide"
        >{$t('crew.close')}</button
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
      {@render trickCards(view.last_trick.cards, false, view.last_trick.winner)}
      <button popovertarget="crew-last-trick" popovertargetaction="hide"
        >{$t('crew.close')}</button
      >
    </section>
  {/if}
</div>

<style>
  .crew-table {
    /* One legible 72 × 101px card at default text size, in every context.
       Tight layouts scroll within the viewport instead of shrinking cards. */
    --arena-card-width: 4.5rem;
    --arena-card-radius: 0.5rem;
    min-width: 0;
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: minmax(0, 1fr) auto;
    gap: 1rem;
    padding: 5rem max(1rem, env(safe-area-inset-right)) 1rem
      max(1rem, env(safe-area-inset-left));
    color: var(--arena-text);
    background: var(--arena-canvas);
  }
  .arena {
    min-width: 0;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    /* A side player needs two card heights plus their name and rack spacing. */
    grid-template-rows: auto minmax(
        calc(var(--arena-card-width) * 2.8 + 4rem),
        1fr
      );
    align-items: center;
    gap: 0.5rem;
    overflow: auto;
  }
  .opponents {
    display: contents;
  }
  .opponent {
    min-width: calc(2 * var(--arena-card-width) + 0.5rem);
  }
  .opponent:first-child {
    grid-area: 2 / 1;
  }
  .arena[data-count='2'] .opponent:first-child {
    grid-area: 1 / 1 / 2 / 3;
  }
  .arena[data-count='2'] .opponent:nth-child(2) {
    grid-area: 1 / 3 / 2 / 5;
  }
  .arena[data-count='3'] .opponent:nth-child(2) {
    grid-area: 1 / 2 / 2 / 4;
  }
  .arena[data-count='3'] .opponent:nth-child(3) {
    grid-area: 2 / 4;
  }
  .arena[data-count='4'] .opponent:nth-child(2) {
    grid-area: 1 / 2;
  }
  .arena[data-count='4'] .opponent:nth-child(3) {
    grid-area: 1 / 3;
  }
  .arena[data-count='4'] .opponent:nth-child(4) {
    grid-area: 2 / 4;
  }
  .arena[data-count='2'] .trick {
    grid-column: 1 / -1;
  }
  .trick {
    grid-area: 2 / 2 / 3 / 4;
    min-width: 0;
    min-height: calc(var(--arena-card-width) * 1.4 + 6rem);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
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
  .history-cards {
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
  .winning :global(.playing-card) {
    outline: 3px solid var(--arena-primary);
    outline-offset: 2px;
    box-shadow: 0 0 0 2px var(--arena-card-face);
  }
  .own-zone {
    min-width: 0;
    display: grid;
    gap: 0.5rem;
  }
  .status {
    text-align: center;
    margin: 0;
    font-size: 0.9rem;
    min-height: 2.5rem;
    display: grid;
    place-content: center;
  }
  .status.your-turn {
    width: fit-content;
    margin-inline: auto;
    padding: 0.5rem 1.25rem;
    border-radius: 999px;
    background: var(--arena-primary);
    color: var(--arena-on-primary);
    font-size: 1rem;
    font-weight: 800;
    box-shadow: 0 2px 0 #1e553d25;
  }
  .own-cards {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 1rem;
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
    background: var(--crew-black);
    border: 2px solid var(--arena-card-face);
  }
  .hand-scroll {
    min-width: 0;
    overflow-x: auto;
    padding: 0.5rem;
  }
  .card-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: max-content;
    margin: auto;
    padding-bottom: 0.25rem;
  }
  .controls,
  .declarations {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
  }
  .controls > button {
    min-height: var(--arena-control-height);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
    border-color: var(--arena-border);
  }
  .controls :global(button) {
    font-size: 0.875rem;
    padding: 0.625rem 0.75rem;
    white-space: nowrap;
  }
  .controls > .history {
    display: grid;
    place-items: center;
    width: 3rem;
    padding: 0.5rem;
  }
  .table-popover {
    width: min(28rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    padding: 1.5rem;
    overflow: auto;
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    box-shadow: 0 12px 36px #1e553d20;
  }
  .table-popover h2 {
    margin-top: 0;
  }
  .table-popover > button {
    margin-top: 1rem;
    min-height: 2.75rem;
  }
  .declarations {
    flex-wrap: wrap;
  }
  @media (max-width: 40rem) {
    .crew-table {
      padding: 0.5rem 0.5rem calc(4.5rem + env(safe-area-inset-bottom));
      gap: 0.5rem;
    }
    .arena {
      gap: 0.25rem;
    }
    .own-cards {
      gap: 0.5rem;
    }
    .own-communication {
      gap: 0.5rem;
    }
    .trick-cards {
      gap: 0.25rem;
    }
    .controls {
      gap: 0.375rem;
    }
  }
  @media (max-height: 500px) and (min-width: 40.01rem) {
    .crew-table {
      padding: 0.5rem 5rem;
      gap: 0.25rem;
    }
    .arena {
      grid-template-columns: repeat(var(--opponents), minmax(0, 1fr));
    }
    .arena[data-count] .opponent:nth-child(n) {
      grid-column: auto;
      grid-row: 1;
    }
    .arena[data-count] .trick {
      grid-column: 1 / -1;
      grid-row: 2;
    }
    .trick-title,
    .winner {
      margin: 0.125rem 0;
    }
    .own-zone {
      gap: 0.25rem;
    }
    .hand-scroll {
      padding: 0.375rem;
    }
  }
  @media (max-width: 40rem) and (orientation: portrait) {
    .arena {
      display: flex;
      flex-direction: column;
      align-items: stretch;
      gap: 1rem;
    }
    .opponents {
      display: flex;
      flex-wrap: nowrap;
      justify-content: start;
      min-width: 0;
      gap: 1rem;
      overflow-x: auto;
      flex-shrink: 0;
      padding-bottom: 1rem;
    }
    .opponent {
      flex: 1 0 calc(2 * var(--arena-card-width) + 0.5rem);
    }
    .trick {
      flex-shrink: 0;
    }
    .own-cards {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      grid-template-areas: 'communication tricks' 'hand hand';
    }
    .own-communication {
      grid-area: communication;
      justify-content: start;
    }
    .own-tricks {
      grid-area: tricks;
      justify-self: end;
    }
    .hand-scroll {
      grid-area: hand;
    }
  }
  @media (max-width: 22.5rem) {
    .arena {
      grid-template-columns: repeat(var(--opponents), minmax(0, 1fr));
    }
    .arena[data-count] .opponent:nth-child(n) {
      grid-column: auto;
      grid-row: 1;
    }
    .arena[data-count] .trick {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
</style>
