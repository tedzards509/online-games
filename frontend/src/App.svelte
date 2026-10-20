<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from './lib/i18n';
  import {
    createRoom,
    downloadReplay,
    joinRoom,
    loadMembership,
    saveMembership,
  } from './lib/api';
  import { RoomConnection } from './lib/connection.svelte';
  import { games, type Membership, type Snapshot } from './lib/types';
  import GameBoard from './components/GameBoard.svelte';
  import GameConfig from './components/GameConfig.svelte';
  import RoomSettings from './components/RoomSettings.svelte';
  import LandingPage from './components/LandingPage.svelte';
  import LanguageSettings from './components/LanguageSettings.svelte';

  function pathCode(): string {
    return (
      /^\/room\/([A-Z2-9]{6})\/?$/i
        .exec(location.pathname)?.[1]
        .toUpperCase() ?? ''
    );
  }
  const initialCode = pathCode();
  let code = $state(initialCode);
  let member = $state<Membership | null>(
    initialCode ? loadMembership(initialCode) : null,
  );
  let connection = $state<RoomConnection | null>(null);
  let nickname = $state(localStorage.getItem('arena-nickname') ?? '');
  let joinCode = $state(initialCode);
  let busy = $state(false);
  let error = $state('');
  let entryError = $state<string | null>(null);
  let copied = $state(false);
  const room = $derived(connection?.room);
  const game = $derived(room?.game);
  const host = $derived(room?.you === room?.host);
  const active = $derived(game != null && game.outcome == null);
  const crewTable = $derived(game?.config.game === 'the_crew');
  const disabled = $derived(
    connection?.pending || connection?.status !== 'connected',
  );

  function reorder(index: number, direction: number) {
    if (!room) return;
    const seats = [...room.seating];
    [seats[index], seats[index + direction]] = [
      seats[index + direction],
      seats[index],
    ];
    connection?.seat(seats);
  }

  function navigate(next: string): void {
    history.pushState(null, '', next ? `/room/${next}` : '/');
    syncLocation();
  }
  function syncLocation(): void {
    code = pathCode();
    joinCode = code;
    member = code ? loadMembership(code) : null;
    error = '';
    entryError = null;
  }
  onMount(() => {
    window.addEventListener('popstate', syncLocation);
    return () => window.removeEventListener('popstate', syncLocation);
  });
  $effect(() => {
    const current = member;
    if (!current) {
      connection = null;
      return;
    }
    const active = new RoomConnection(current);
    connection = active;
    return () => active.close();
  });

  async function enter(create: boolean): Promise<void> {
    if (busy) return;
    busy = true;
    error = '';
    entryError = null;
    try {
      const target = joinCode.trim().toUpperCase();
      const existing = !create ? loadMembership(target) : null;
      const membership =
        existing ??
        (await (create ? createRoom(nickname) : joinRoom(target, nickname)));
      saveMembership(membership);
      localStorage.setItem('arena-nickname', nickname.trim());
      navigate(membership.code);
    } catch (cause) {
      // The HTTP API supplies English messages; translate expected entry errors.
      const messages: Record<string, string> = {
        'Room or replay not found': 'roomNotFound',
        'Nickname already taken': 'nicknameTaken',
        'Use a nickname of 1–24 characters': 'nicknameInvalid',
      };
      entryError =
        (cause instanceof Error && messages[cause.message]) || 'enterFailed';
    } finally {
      busy = false;
    }
  }

  async function copyCode(): Promise<void> {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
    } catch {
      error = 'Could not copy code';
    }
  }

  async function replay(id: string): Promise<void> {
    if (!member) return;
    try {
      await downloadReplay(member.code, id, member.token);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Download failed';
    }
  }
</script>

{#snippet replayList(room: Snapshot)}
  {#if room.replays.length}<details class="replays">
      <summary>Replays ({room.replays.length})</summary>
      {#each room.replays as archive, index}<button
          class="quiet"
          onclick={() => replay(archive.id)}
          >Game {index + 1} · {games[archive.config.game]} ↓</button
        >{/each}
    </details>{/if}
{/snippet}

<svelte:head
  ><title>{code ? `${code} · ` : ''}Friends' Arena</title></svelte:head
>

<main
  class:entry-shell={!member}
  class:crew-shell={crewTable}
  lang={member ? 'en' : undefined}
>
  <LanguageSettings />
  {#if member && !crewTable}<header>
      <a
        class="brand"
        href="/"
        onclick={(event) => {
          event.preventDefault();
          navigate('');
        }}>Friends' Arena</a
      >
      {#if code}<button
          class="quiet"
          onclick={copyCode}
          aria-label="Copy room code">{code} {copied ? '✓' : '⧉'}</button
        >{/if}
    </header>{/if}

  {#if error || connection?.error}<p class="error" role="alert">
      {error || connection?.error}
    </p>{/if}
  {#if entryError}<p class="error entry-error" role="alert">
      {$t(entryError)}
    </p>{/if}

  {#if !member}
    <LandingPage bind:nickname bind:joinCode invited={!!code} {busy} {enter} />
  {:else if !room}
    <p role="status">
      {connection?.status === 'offline'
        ? 'Disconnected. Reconnecting…'
        : 'Connecting…'}
    </p>
  {:else}
    <section class="room" aria-label="Game room">
      {#if crewTable && connection?.status !== 'connected'}<p
          class="connection-notice"
          role="status"
        >
          {$t('crew.reconnecting')}
        </p>{/if}
      {#if crewTable && host}<button
          class="room-trigger"
          popovertarget="room-controls"
          aria-label={$t('crew.roomControls')}
          title={$t('crew.roomControls')}
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.7"
            aria-hidden="true"
            ><path d="M4 6h16M4 12h16M4 18h16" /><circle
              cx="9"
              cy="6"
              r="2"
              fill="var(--arena-surface)"
            /><circle
              cx="15"
              cy="12"
              r="2"
              fill="var(--arena-surface)"
            /><circle cx="9" cy="18" r="2" fill="var(--arena-surface)" /></svg
          >
        </button>{/if}
      {#if !crewTable || host}
        <div
          id="room-controls"
          popover={crewTable ? 'auto' : undefined}
          class:crew-controls={crewTable}
        >
          {#if crewTable}<div class="menu-heading">
              <button onclick={copyCode} aria-label="Copy room code"
                >{code} {copied ? '✓' : '⧉'}</button
              ><button popovertarget="room-controls" popovertargetaction="hide"
                >{$t('crew.close')}</button
              >
            </div>{/if}
          <div class="room-title">
            <h1>{game ? games[game.config.game] : 'Room'}</h1>
            <span class="muted" role="status"
              >{connection?.status === 'connected' ? '' : 'Reconnecting…'}</span
            >
          </div>
          <div class="room-setup">
            <ul aria-label="Members">
              {#each room.members as member}
                <li>
                  {member.nickname}{member.id === room.you
                    ? ' · you'
                    : ''}{member.id === room.host ? ' · host' : ''}
                </li>
              {/each}
            </ul>
            {#if room.config}<p>
                Selected game: {games[room.config.game]}
              </p>{/if}
            {#if host}
              <RoomSettings
                config={room.config}
                players={room.seating.map((id) => room.members[id].nickname)}
                disabled={disabled || active}
                change={(config) => connection?.configure(config)}
              />
            {:else if room.config}
              <GameConfig config={room.config} />
            {:else}<p>Waiting for host to choose a game</p>{/if}
            <ol aria-label="Seating">
              {#each room.seating as member, index}
                <li>
                  {room.members[member].nickname}
                  {#if host}
                    <button
                      aria-label={`Move ${room.members[member].nickname} up`}
                      disabled={disabled || active || index === 0}
                      onclick={() => reorder(index, -1)}>↑</button
                    >
                    <button
                      aria-label={`Move ${room.members[member].nickname} down`}
                      disabled={disabled ||
                        active ||
                        index === room.seating.length - 1}
                      onclick={() => reorder(index, 1)}>↓</button
                    >
                    <button
                      disabled={disabled || active}
                      onclick={() =>
                        connection?.seat(
                          room.seating.filter((m) => m !== member),
                        )}>Bench {room.members[member].nickname}</button
                    >
                  {/if}
                </li>
              {/each}
            </ol>
            <ul aria-label="Bench">
              {#each room.members.filter((m) => !room.seating.includes(m.id)) as member}
                <li>
                  {member.nickname}{#if host}<button
                      disabled={disabled || active}
                      onclick={() =>
                        connection?.seat([...room.seating, member.id])}
                      >Seat {member.nickname}</button
                    >{/if}
                </li>
              {/each}
            </ul>
          </div>
          {#if !active && host}
            <button
              class="primary"
              disabled={disabled ||
                !room.config ||
                room.seating.length <
                  (room.config?.game === 'the_crew' ||
                  room.config?.game === 'tempel_des_schreckens'
                    ? 3
                    : 2)}
              onclick={() => connection?.start()}
              >{game ? 'Play again' : 'Start game'}</button
            >
          {/if}
          {#if game}
            {#if host}<button {disabled} onclick={() => connection?.abort()}
                >{active ? 'Abort game' : 'Return to setup'}</button
              >{/if}
            {#if !crewTable}<ul class="players" aria-label="Players">
                {#each game.players as player, index}
                  <li
                    class:active={active && game.turn.players.includes(index)}
                  >
                    {player.nickname}{index === game.you ? ' · you' : ''}
                  </li>
                {/each}
              </ul>{/if}
            {#if game.outcome}
              <h2 role="status">
                {game.outcome.winners.length
                  ? `${game.outcome.winners.map((p) => game.players[p].nickname).join(', ')} won`
                  : game.config.game === 'the_crew'
                    ? 'Lost'
                    : 'Draw'}
              </h2>
              {#if game.outcome.scores}<p>
                  {game.outcome.scores
                    .map((score, p) => `${game.players[p].nickname}: ${score}`)
                    .join(' · ')}
                </p>{/if}
            {:else}
              {#if game.turn.players.length && !crewTable}<p
                  class="turn"
                  role="status"
                >
                  {game.you != null && game.turn.players.includes(game.you)
                    ? 'Your turn'
                    : `${game.turn.players.map((player) => game.players[player].nickname).join(', ') || 'Nobody'}'s turn`}
                </p>{/if}
              {#if game.can_finish}
                <div class="controls" aria-label="Game result">
                  <button
                    disabled={connection?.pending ||
                      connection?.status !== 'connected'}
                    onclick={() => connection?.finish(true)}>Finish: won</button
                  >
                  <button
                    disabled={connection?.pending ||
                      connection?.status !== 'connected'}
                    onclick={() => connection?.finish(false)}
                    >Finish: lost</button
                  >
                </div>
              {/if}
            {/if}
          {/if}
          {#if crewTable}{@render replayList(room)}{/if}
        </div>
      {/if}
      {#if game}<GameBoard
          room={game}
          roomInfo={{ code: room.code, host: room.host }}
          disabled={connection?.pending || connection?.status !== 'connected'}
          move={(action) => connection?.move(action)}
        />
      {/if}
      {#if !crewTable}{@render replayList(room)}{/if}
    </section>
  {/if}
</main>

<style>
  main.crew-shell {
    position: fixed;
    inset: 0;
    width: 100%;
    max-width: none;
    height: 100vh;
    height: 100dvh;
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: var(--arena-canvas);
    display: grid;
  }
  .crew-shell .room {
    min-height: 0;
    display: grid;
  }
  .room-trigger {
    position: fixed;
    top: var(--arena-space-3);
    left: max(var(--arena-space-3), env(safe-area-inset-left));
    z-index: 10;
    display: grid;
    place-items: center;
    width: var(--arena-control-height);
    height: var(--arena-control-height);
    padding: 0.75rem;
    border: 1px solid var(--arena-border);
    border-radius: 50%;
    background: var(--arena-surface);
    color: var(--arena-text);
  }
  .room-trigger svg {
    width: 100%;
    height: 100%;
  }
  .crew-controls {
    position: fixed;
    inset: 5.25rem auto auto var(--arena-space-3);
    margin: 0;
    width: min(30rem, calc(100vw - 3rem));
    max-height: calc(100dvh - 7rem);
    padding: var(--arena-space-3);
    overflow: auto;
    background: var(--arena-surface);
    color: var(--arena-text);
    border: 1px solid var(--arena-border);
    border-radius: var(--arena-panel-radius);
    box-shadow: 0 12px 36px #1e553d20;
  }
  .menu-heading {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  .crew-shell .error,
  .connection-notice {
    position: fixed;
    top: 0.75rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9;
    max-width: calc(100% - 10rem);
    margin: 0;
    padding: 0.5rem 0.75rem;
    border-radius: var(--arena-control-radius);
    background: var(--arena-surface);
    font-size: 0.85rem;
  }
  .room-trigger:focus-visible {
    outline-color: var(--arena-focus);
  }
  @media (max-width: 40rem) {
    .room-trigger {
      top: auto;
      left: max(var(--arena-space-2), env(safe-area-inset-left));
      bottom: max(var(--arena-space-2), env(safe-area-inset-bottom));
    }
    .crew-controls {
      inset: auto auto calc(5rem + env(safe-area-inset-bottom))
        var(--arena-space-2);
      width: calc(100vw - 2rem);
    }
    .crew-shell .error,
    .connection-notice {
      max-width: calc(100% - 2rem);
    }
  }
  main.entry-shell {
    max-width: none;
    min-height: 100svh;
    display: flex;
    flex-direction: column;
    padding: 6rem var(--arena-space-3);
    background: var(--arena-canvas);
    color: var(--arena-text);
  }
  .entry-error {
    width: min(100%, var(--arena-entry-width));
    margin: 0 auto var(--arena-space-3);
  }
  @media (max-width: 40rem) {
    main.entry-shell {
      padding: var(--arena-space-4) var(--arena-space-2) 6rem;
    }
  }
</style>
