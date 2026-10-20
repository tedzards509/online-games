<script lang="ts">
  import type { Schemas, Snapshot } from '../../lib/types';
  import { t } from '../../lib/i18n';
  import CardBack from '../cards/CardBack.svelte';
  import CommunicationSlot from './CommunicationSlot.svelte';
  import TrickPile from './TrickPile.svelte';

  let {
    player,
    member,
    view,
    roomInfo,
    finished,
  }: {
    player: number;
    member: Schemas['MemberView'];
    view: Schemas['CrewView'];
    roomInfo?: Pick<Snapshot, 'code' | 'host'>;
    finished: boolean;
  } = $props();
  const communication = $derived(view.communications[player]);
  const commander = $derived(view.commander === player);
  const currentCard = $derived(
    view.trick.find((play) => play.player === player)?.card,
  );
  const lastCard = $derived(
    view.last_trick?.cards.find((play) => play.player === player)?.card,
  );
  function cardName(card: Schemas['CrewCard']): string {
    return $t('crew.card', {
      values: {
        color: $t(`crew.colors.${card.color}`),
        number: String(card.number),
      },
    });
  }
</script>

<section
  class="player"
  aria-label={member.nickname}
  class:next={!finished && !view.awaiting_result && view.next_player === player}
>
  <button
    class="info"
    popovertarget={`crew-info-${player}`}
    aria-label={$t('crew.infoAbout', { values: { name: member.nickname } })}
    ><span title={member.nickname}>{member.nickname}</span><svg
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      aria-hidden="true"
      ><circle cx="10" cy="10" r="8" /><path d="M10 9v5" /><circle
        cx="10"
        cy="6"
        r="0.6"
        fill="currentColor"
      /></svg
    ></button
  >
  <div class="backs" aria-hidden="true">
    {#each Array(view.counts[player]) as _}<span class="back"><CardBack /></span
      >{/each}
  </div>
  <div class="rack">
    <CommunicationSlot
      {communication}
      played={view.communication_played[player]}
      name={member.nickname}
    />
    <TrickPile count={view.tricks_won[player]} name={member.nickname} />
  </div>
  {#if commander}<span
      class="commander"
      role="img"
      aria-label={$t('crew.commander')}
      title={$t('crew.commander')}
    ></span>{/if}
</section>

<section
  popover
  id={`crew-info-${player}`}
  class="player-info"
  aria-label={$t('crew.infoAbout', { values: { name: member.nickname } })}
>
  <h2>{member.nickname}</h2>
  <dl>
    {#if roomInfo}<dt>{$t('roomCode')}</dt>
      <dd>{roomInfo.code}</dd>{/if}
    <dt>{$t('crew.memberId')}</dt>
    <dd>{member.id}</dd>
    <dt>{$t('crew.seat')}</dt>
    <dd>{player + 1}</dd>
    {#if roomInfo}<dt>{$t('crew.roomRole')}</dt>
      <dd>
        {$t(roomInfo.host === member.id ? 'crew.host' : 'crew.member')}
      </dd>{/if}
    <dt>{$t('crew.cardsRemaining')}</dt>
    <dd>{view.counts[player]}</dd>
    <dt>{$t('crew.tricksWon')}</dt>
    <dd>{view.tricks_won[player]}</dd>
    <dt>{$t('crew.commanderName')}</dt>
    <dd>{$t(commander ? 'crew.yes' : 'crew.no')}</dd>
    <dt>{$t('crew.nextToPlay')}</dt>
    <dd>
      {$t(
        !finished && !view.awaiting_result && view.next_player === player
          ? 'crew.yes'
          : 'crew.no',
      )}
    </dd>
    <dt>{$t('crew.communication')}</dt>
    <dd>
      {#if communication}
        {$t('crew.card', {
          values: {
            color: $t(`crew.colors.${communication.card.color}`),
            number: String(communication.card.number),
          },
        })} · {$t(`crew.declarations.${communication.declaration}`)} · {$t(
          view.communication_played[player]
            ? 'crew.communicationPlayed'
            : 'crew.stillInHand',
        )}
      {:else}{$t('crew.communicationReady')}{/if}
    </dd>
    <dt>{$t('crew.currentTrick')}</dt>
    <dd>{currentCard ? cardName(currentCard) : $t('crew.notPlayed')}</dd>
    {#if lastCard}<dt>{$t('crew.lastTrick')}</dt>
      <dd>{cardName(lastCard)}</dd>
      <dt>{$t('crew.wonLastTrick')}</dt>
      <dd>
        {$t(view.last_trick?.winner === player ? 'crew.yes' : 'crew.no')}
      </dd>{/if}
  </dl>
  <button popovertarget={`crew-info-${player}`} popovertargetaction="hide"
    >{$t('crew.close')}</button
  >
</section>

<style>
  .player {
    min-width: 0;
    text-align: center;
  }
  .info > span {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .next .info > span {
    text-decoration: underline;
    text-decoration-thickness: 2px;
    text-underline-offset: 0.3rem;
  }
  button.info {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    width: fit-content;
    max-width: 100%;
    margin-inline: auto;
    min-height: 2.75rem;
    padding: 0 0.5rem;
    border: 0;
    border-radius: var(--arena-control-radius);
    background: transparent;
    color: var(--arena-text);
    font-weight: 800;
    font-size: 0.875rem;
  }
  .info svg {
    flex-shrink: 0;
    width: 1.125rem;
    height: 1.125rem;
  }
  .rack {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .commander {
    display: block;
    margin: 0.5rem auto 0;
    width: calc(var(--arena-card-width) * 0.75);
    aspect-ratio: 1;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--crew-black);
    border: 2px solid var(--arena-card-face);
  }
  .backs {
    display: flex;
    justify-content: center;
    min-height: calc(var(--arena-card-width) * 1.4);
  }
  .back {
    width: 3px;
    flex: 0 0 3px;
  }
  .back:last-child {
    width: var(--arena-card-width);
    flex-basis: var(--arena-card-width);
  }
  .player-info {
    width: min(24rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    padding: var(--arena-space-3);
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-panel-radius);
    background: var(--arena-surface);
    color: var(--arena-text);
    overflow-y: auto;
    box-shadow: 0 12px 36px #1e553d20;
  }
  h2 {
    margin: 0 0 1rem;
    overflow-wrap: anywhere;
  }
  dl {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 0.75rem;
    text-align: left;
    font-size: 0.9rem;
  }
  dt {
    font-weight: 700;
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .player-info button {
    min-height: 2.75rem;
    background: var(--arena-input);
    border-color: var(--arena-border);
  }
  button:focus-visible {
    outline-color: var(--arena-focus);
  }
</style>
