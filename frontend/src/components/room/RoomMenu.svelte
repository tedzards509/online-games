<script lang="ts">
  import { t } from '../../lib/i18n';
  import type { RoomConnection } from '../../lib/connection.svelte';
  import type { GameSnapshot, Snapshot } from '../../lib/types';
  import Avatar from '../table/Avatar.svelte';
  import Button from '../ui/Button.svelte';
  import Corner from '../ui/Corner.svelte';
  import Popover from '../ui/Popover.svelte';
  import GameStatus from './GameStatus.svelte';
  import HostMark from './HostMark.svelte';
  import ReplayList from './ReplayList.svelte';
  import RoomCode from './RoomCode.svelte';

  /** Room controls during a game, opposite the settings button. Everyone sees the
   * room code, members, and replays; the host also gets the game commands.
   * `withStatus` repeats the game status for boards that hide the page header. */
  let {
    room,
    game,
    connection,
    disabled,
    withStatus,
    download,
    onerror,
    onresults,
  }: {
    room: Snapshot;
    game: GameSnapshot;
    connection: RoomConnection;
    disabled: boolean;
    withStatus: boolean;
    download: (id: string) => void;
    onerror: (message: string) => void;
    /** Reopens the end screen, which holds the host's commands once a game ends. */
    onresults: () => void;
  } = $props();
  const host = $derived(room.you === room.host);
</script>

<Corner placement="start">
  <Button
    variant="quiet"
    square
    popovertarget="room-controls"
    aria-label={$t('room.controls')}
    title={$t('room.controls')}
  >
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      aria-hidden="true"
      ><path d="M4 6h16M4 12h16M4 18h16" /><circle
        cx="9"
        cy="6"
        r="2"
        fill="var(--arena-surface)"
      /><circle cx="15" cy="12" r="2" fill="var(--arena-surface)" /><circle
        cx="9"
        cy="18"
        r="2"
        fill="var(--arena-surface)"
      /></svg
    >
  </Button>
</Corner>

<Popover id="room-controls" label={$t('room.controls')} placement="start">
  <div class="menu">
    <div class="heading">
      <RoomCode code={room.code} {onerror} />
      <Button
        variant="quiet"
        size="small"
        popovertarget="room-controls"
        popovertargetaction="hide">{$t('common.close')}</Button
      >
    </div>
    {#if withStatus}<GameStatus {game} showTurn={false} />{/if}
    {#if game.outcome}
      <div class="commands">
        <Button onclick={onresults}>{$t('room.results')}</Button>
      </div>
    {:else if host}
      <div class="commands">
        {#if game.can_finish}
          <Button {disabled} onclick={() => connection.finish(true)}
            >{$t('room.finishWon')}</Button
          >
          <Button {disabled} onclick={() => connection.finish(false)}
            >{$t('room.finishLost')}</Button
          >
        {/if}
        <Button
          variant="secondary"
          {disabled}
          onclick={() => connection.abort()}>{$t('room.abort')}</Button
        >
      </div>
    {/if}
    <ul aria-label={$t('room.members')}>
      {#each room.members as member (member.id)}
        <li>
          <Avatar name={member.nickname} colorIndex={member.id} />
          {member.nickname}{member.id === room.you
            ? ` · ${$t('common.you')}`
            : ''}
          {#if member.id === room.host}<HostMark />{/if}
        </li>
      {/each}
    </ul>
    <ReplayList replays={room.replays} {download} />
  </div>
</Popover>

<style>
  .menu {
    display: grid;
    gap: var(--arena-space-2);
  }
  .heading,
  .commands {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 0.75rem;
  }
  .commands {
    justify-content: start;
  }
  ul {
    display: grid;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    font-weight: 700;
  }
</style>
