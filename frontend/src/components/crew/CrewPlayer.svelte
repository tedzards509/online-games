<script lang="ts">
  import type { Schemas, Snapshot } from '../../lib/types';
  import { t } from '../../lib/i18n';
  import OpponentSeat from '../table/OpponentSeat.svelte';
  import CommunicationSlot from './CommunicationSlot.svelte';
  import TrickPile from './TrickPile.svelte';
  import { crewColors } from './cards';

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

<OpponentSeat
  name={member.nickname}
  colorIndex={player}
  cards={view.counts[player]}
  active={!finished && !view.awaiting_result && view.next_player === player}
>
  {#snippet markers()}
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
        style:background={crewColors.black}
        role="img"
        aria-label={$t('crew.commander')}
        title={$t('crew.commander')}
      ></span>{/if}
  {/snippet}
  {#snippet info()}
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
  {/snippet}
</OpponentSeat>

<style>
  /* Communication and won tricks side by side; the commander below. */
  .rack {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: calc(var(--arena-card-width) * 0.1);
    flex-basis: 100%;
  }
  .commander {
    width: calc(var(--arena-card-width) * 0.75);
    aspect-ratio: 1;
    flex-shrink: 0;
    border-radius: 50%;
    border: var(--arena-line);
  }
  dl {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 0.75rem;
    margin: 0 0 1rem;
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
</style>
