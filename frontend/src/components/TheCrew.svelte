<script lang="ts">
  import type { Action, Schemas, Snapshot } from '../lib/types';
  import { locale, t } from '../lib/i18n';
  import Button from './ui/Button.svelte';
  import CrewCard from './crew/CrewCard.svelte';
  import CrewPlayer from './crew/CrewPlayer.svelte';
  import CardFan from './crew/CardFan.svelte';
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
  }: {
    view: Schemas['CrewView'];
    actions: Action[];
    players: Schemas['MemberView'][];
    you: number;
    move: (action: Action) => void;
    finished: boolean;
    disabled: boolean;
    roomInfo?: Pick<Snapshot, 'code' | 'host'>;
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

  // Relative seating runs clockwise from the viewer's left, preserving play order.
  const positions: Record<number, [number, number][]> = {
    2: [
      [22, 5],
      [78, 5],
    ],
    3: [
      [14, 32],
      [50, 0],
      [86, 32],
    ],
    4: [
      [14, 32],
      [32, 0],
      [68, 0],
      [86, 32],
    ],
  };

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
    if (!action || disabled || finished) return;
    selected = null;
    move(action);
  }
  /** Stable visual scatter, unrelated to game randomness or the hidden deal. */
  function scatter(played: Schemas['CrewPlay']): string {
    const value =
      played.player * 17 + played.card.number * 7 + played.card.color.length;
    return `translate(${(value % 7) - 3}px, ${(value % 13) - 6}px) rotate(${(value % 11) - 5}deg)`;
  }
</script>

<div
  class="crew-table"
  style:--crew-black={crewColors.black}
  lang={$locale === 'de-cool' ? 'de' : $locale}
>
  <div class="arena">
    {#each opponents as player, index (player)}
      <div
        class="opponent"
        class:side={positions[opponents.length][index][1] === 32}
        style:left={`${positions[opponents.length][index][0]}%`}
        style:--row={`${positions[opponents.length][index][1]}%`}
      >
        <CrewPlayer
          {player}
          member={players[player]}
          {view}
          {roomInfo}
          {finished}
        />
      </div>
    {/each}

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
            <figcaption>{players[played.player].nickname}</figcaption>
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
    <div class="own-hand">
      <p class="status" role="status">
        {#if finished}{$t('crew.finished')}
        {:else if view.awaiting_result}{$t('crew.awaitingResult')}
        {:else}{$t(view.trick.length ? 'crew.playsNext' : 'crew.leads', {
            values: { name: players[view.next_player].nickname },
          })}{/if}
      </p>
      <section class="hand-scroll" aria-label={$t('crew.yourHand')}>
        <CardFan count={view.hand.length} selected={selectedIndex}>
          {#snippet card(index)}
            <div class="hand-card" class:unplayable={!play(index)}>
              <CrewCard
                card={view.hand[index]}
                label={$t('crew.selectCard', {
                  values: { card: cardName(view.hand[index]) },
                })}
                selected={selectedIndex === index}
                disabled={disabled || finished}
                onclick={() =>
                  (selected =
                    selectedIndex === index ? null : cardKey(view.hand[index]))}
              />
            </div>
          {/snippet}
        </CardFan>
      </section>
    </div>
    <div class="own-tricks">
      <TrickPile count={view.tricks_won[you]} name={players[you].nickname} />
    </div>
  </div>

  <div class="controls">
    <p class="selection">
      {selectedIndex >= 0
        ? cardName(view.hand[selectedIndex])
        : $t('crew.selectPrompt')}
    </p>
    <Button
      disabled={disabled || finished || !play(selectedIndex)}
      onclick={() => submit(play(selectedIndex))}>{$t('crew.playCard')}</Button
    >
    {#if communicationAvailable && !finished}
      <details class="communicate">
        <summary>{$t('crew.communicateCard')}</summary>
        <div class="declarations">
          {#each ['highest', 'only', 'lowest'] as declaration}
            <Button
              variant="secondary"
              disabled={disabled ||
                !communicate(declaration as Schemas['CrewDeclaration'])}
              onclick={() =>
                submit(communicate(declaration as Schemas['CrewDeclaration']))}
              >{$t(`crew.declarations.${declaration}`)}</Button
            >
          {/each}
        </div>
      </details>
    {/if}
    {#if view.last_trick && view.trick.length}
      <details class="last-trick">
        <summary>{$t('crew.lastTrick')}</summary>
        <p>
          {$t('crew.trickWinner', {
            values: { name: players[view.last_trick.winner].nickname },
          })}
        </p>
        {#each view.last_trick.cards as played}<p>
            {players[played.player].nickname}: {cardName(played.card)}
          </p>{/each}
      </details>
    {/if}
  </div>
</div>

<style>
  .crew-table {
    /* Public markers use compact cards; the hand keeps readable, touch-sized faces. */
    --crew-small-card: clamp(2.25rem, 4vw, 3.25rem);
    --crew-trick-card: clamp(2.75rem, 5vw, 4.25rem);
    --arena-card-radius: 0.5rem;
    color: var(--arena-text);
    background: var(--arena-canvas);
    padding: 0.75rem 0 1rem;
  }
  .arena {
    position: relative;
    min-height: max(20rem, calc(100svh - 32rem));
  }
  .opponent {
    position: absolute;
    top: var(--row);
    width: 9rem;
    transform: translateX(-50%);
  }
  .trick {
    position: absolute;
    top: 60%;
    left: 50%;
    transform: translateX(-50%);
    width: max-content;
    max-width: 100%;
    text-align: center;
  }
  .trick-title,
  .winner {
    margin: 0 0 0.75rem;
    font-size: 0.8rem;
  }
  .winner {
    margin: 0.75rem 0 0;
  }
  .trick-cards {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.25rem;
    min-height: 6rem;
  }
  figure {
    margin: 0;
    width: var(--crew-trick-card);
  }
  figcaption {
    font-size: 0.7rem;
    margin-top: 0.5rem;
    overflow-wrap: anywhere;
  }
  .own-zone {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 1rem;
  }
  .own-communication {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-left: 1rem;
  }
  .own-tricks {
    padding-right: 1rem;
  }
  .commander {
    width: 1.375rem;
    height: 1.375rem;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--crew-black);
    border: 2px solid var(--arena-card-face);
  }
  .own-hand {
    min-width: 0;
  }
  .status {
    text-align: center;
    margin: 0;
    font-size: 0.9rem;
  }
  .hand-scroll {
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0.5rem 1rem 0;
  }
  .hand-card.unplayable {
    opacity: 0.58;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 0 0.5rem;
  }
  .selection {
    flex-basis: 100%;
    text-align: center;
    margin: 0;
    font-size: 0.85rem;
    min-height: 1.5em;
  }
  summary {
    min-height: 3rem;
    padding: 0.75rem;
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
  }
  summary:focus-visible {
    outline-color: var(--arena-focus);
  }
  .declarations {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
  .communicate[open],
  .last-trick[open] {
    flex-basis: 100%;
    text-align: center;
  }
  .communicate[open] .declarations {
    justify-content: center;
  }
  @media (max-width: 40rem) {
    .arena {
      min-height: 30rem;
    }
    .opponent {
      width: 7.5rem;
    }
    .opponent.side {
      top: 38%;
    }
    .trick {
      top: 72%;
    }
    .trick-cards {
      gap: 0.25rem;
    }
    .own-zone {
      gap: 0.25rem;
    }
    .own-communication {
      flex-direction: column;
      padding-left: 0.5rem;
      gap: 0.5rem;
    }
    .own-tricks {
      padding-right: 0.5rem;
    }
  }
</style>
