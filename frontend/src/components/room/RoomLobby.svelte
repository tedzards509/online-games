<script lang="ts">
  import type { RoomConnection } from '../../lib/connection.svelte';
  import { t } from '../../lib/i18n';
  import type { Snapshot } from '../../lib/types';
  import Avatar from '../table/Avatar.svelte';
  import PlayerBar from '../table/PlayerBar.svelte';
  import Button from '../ui/Button.svelte';
  import Panel from '../ui/Panel.svelte';
  import GameConfig from './GameConfig.svelte';
  import HostMark from './HostMark.svelte';
  import ReplayList from './ReplayList.svelte';
  import RoomCode from './RoomCode.svelte';
  import RoomSettings from './RoomSettings.svelte';

  /** Room setup between games: game choice, rules, seating, and the bench.
   * Only the host edits; everyone else sees the same setup read-only. */
  let {
    room,
    connection,
    disabled,
    download,
    onerror,
  }: {
    room: Snapshot;
    connection: RoomConnection;
    disabled: boolean;
    download: (id: string) => void;
    onerror: (message: string) => void;
  } = $props();

  const host = $derived(room.you === room.host);
  const bench = $derived(
    room.members.filter((member) => !room.seating.includes(member.id)),
  );
  const minimumPlayers = $derived(
    room.config?.game === 'the_crew' ||
      room.config?.game === 'tempel_des_schreckens'
      ? 3
      : 2,
  );

  function move(index: number, direction: -1 | 1): void {
    const seats = [...room.seating];
    [seats[index], seats[index + direction]] = [
      seats[index + direction],
      seats[index],
    ];
    connection.seat(seats);
  }
</script>

<header>
  <h1>{$t('room.title')}</h1>
  <RoomCode code={room.code} {onerror} />
</header>

<div class="lobby">
  <Panel aria-label={$t('room.gameSetup')}>
    {#if host}
      <RoomSettings
        config={room.config}
        players={room.seating.map((id) => room.members[id].nickname)}
        {disabled}
        change={(config) => connection.configure(config)}
      />
    {:else if room.config}
      <h2>{$t(`gameNames.${room.config.game}`)}</h2>
      <GameConfig config={room.config} />
    {:else}
      <p>{$t('room.waitingForGame')}</p>
    {/if}
  </Panel>

  <Panel aria-label={$t('room.members')}>
    <h2>{$t('room.seats')}</h2>
    <ol aria-label={$t('room.seating')}>
      {#each room.seating as id, index (id)}
        {@const name = room.members[id].nickname}
        <li>
          <PlayerBar {name} colorIndex={id} you={id === room.you}>
            {#if id === room.host}<HostMark />{/if}
            {#if host}
              <span class="controls">
                <Button
                  variant="quiet"
                  size="small"
                  square
                  aria-label={$t('room.moveUp', { values: { name } })}
                  disabled={disabled || index === 0}
                  onclick={() => move(index, -1)}>↑</Button
                >
                <Button
                  variant="quiet"
                  size="small"
                  square
                  aria-label={$t('room.moveDown', { values: { name } })}
                  disabled={disabled || index === room.seating.length - 1}
                  onclick={() => move(index, 1)}>↓</Button
                >
                <Button
                  variant="quiet"
                  size="small"
                  aria-label={$t('room.benchPlayer', { values: { name } })}
                  {disabled}
                  onclick={() =>
                    connection.seat(room.seating.filter((m) => m !== id))}
                  >{$t('room.toBench')}</Button
                >
              </span>
            {/if}
          </PlayerBar>
        </li>
      {/each}
    </ol>
    <h2>{$t('room.bench')}</h2>
    <ul aria-label={$t('room.bench')}>
      {#each bench as member (member.id)}
        <li>
          <Avatar name={member.nickname} colorIndex={member.id} />
          <span class="name"
            >{member.nickname}{member.id === room.you
              ? ` · ${$t('common.you')}`
              : ''}</span
          >
          {#if member.id === room.host}<HostMark />{/if}
          {#if host}
            <span class="controls">
              <Button
                variant="quiet"
                size="small"
                aria-label={$t('room.seatPlayer', {
                  values: { name: member.nickname },
                })}
                {disabled}
                onclick={() => connection.seat([...room.seating, member.id])}
                >{$t('room.seat')}</Button
              >
            </span>
          {/if}
        </li>
      {/each}
    </ul>
    {#if host}
      <Button
        size="large"
        disabled={disabled ||
          !room.config ||
          room.seating.length < minimumPlayers}
        onclick={() => connection.start()}>{$t('room.start')}</Button
      >
    {/if}
  </Panel>
</div>

<ReplayList replays={room.replays} {download} />

<style>
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--arena-space-2);
  }
  .lobby {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: start;
    gap: var(--arena-space-3);
  }
  ol,
  ul {
    display: grid;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  ul li {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.4rem 0.75rem 0.4rem 0.4rem;
    font-weight: 700;
  }
  .controls {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }
  @media (max-width: 40rem) {
    .lobby {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
