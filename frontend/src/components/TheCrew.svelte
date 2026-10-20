<script lang="ts">
  import type { Action, Schemas, Snapshot } from '../lib/types';
  import { locale, t } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import CrewCard from './crew/CrewCard.svelte';
  import CrewPlayer from './crew/CrewPlayer.svelte';
  import CommunicationSlot from './crew/CommunicationSlot.svelte';
  import TrickPile from './crew/TrickPile.svelte';
  import { cardKey, crewColors } from './crew/cards';

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
    {#each opponents as player (player)}<div class="opponent">
        <CrewPlayer
          {player}
          member={players[player]}
          {view}
          {roomInfo}
          {finished}
        />
      </div>{/each}

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
      <div class="trick-cards">
        {#each currentCards as played (played.player)}
          <figure style:transform={scatter(played)}>
            <CrewCard card={played.card} width="var(--crew-trick-card)" />
            <figcaption title={players[played.player].nickname}>
              {players[played.player].nickname}
            </figcaption>
          </figure>
        {/each}
      </div>
      {#if !view.trick.length && view.last_trick}<p class="winner">
          {$t('crew.trickWinner', {
            values: { name: players[view.last_trick.winner].nickname },
          })}
        </p>{/if}
    </section>
  </div>
  <div class="own-zone">
    <p class="status" role="status">
      {#if finished}{$t(won ? 'crew.won' : 'crew.lost')}
      {:else if view.awaiting_result}{$t('crew.awaitingResult')}
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
      <TrickPile count={view.tricks_won[you]} name={players[you].nickname} />
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
      {#each view.last_trick.cards as played}<p>
          {players[played.player].nickname}: {cardName(played.card)}
        </p>{/each}
      <button popovertarget="crew-last-trick" popovertargetaction="hide"
        >{$t('crew.close')}</button
      >
    </section>
  {/if}
</div>

<style>
  .crew-table {
    /* Rows remain inside the viewport; only the hand and popovers can scroll. */
    --crew-small-card: clamp(2.25rem, 4vw, 2.75rem);
    --crew-trick-card: clamp(2rem, 5vw, 4rem);
    --arena-card-width: clamp(3.25rem, 6vw, 5.5rem);
    --arena-card-radius: 0.5rem;
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
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    grid-template-rows: auto minmax(0, 1fr);
    align-items: center;
    gap: 0.5rem;
  }
  .opponent {
    min-width: 0;
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
    min-height: 0;
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
    justify-content: center;
    gap: 0.75rem;
    padding: 0.25rem;
  }
  figure {
    margin: 0;
    width: var(--crew-trick-card);
  }
  figcaption {
    font-size: 0.75rem;
    margin-top: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
    width: 1.375rem;
    height: 1.375rem;
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
      flex-direction: column;
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
      --arena-card-width: 2.75rem;
      --crew-small-card: 1.5rem;
      --crew-back-width: 0.75rem;
      --crew-trick-card: 2rem;
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
