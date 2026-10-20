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
  import { games, type Membership } from './lib/types';
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
  const crewTable = $derived(
    game?.config.game === 'the_crew' && game.you != null,
  );
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

<svelte:head
  ><title>{code ? `${code} · ` : ''}Friends' Arena</title></svelte:head
>

<main
  class:entry-shell={!member}
  class:crew-shell={crewTable}
  lang={member ? 'en' : undefined}
>
  <LanguageSettings />
  {#if member}<header>
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
      <div class="room-title">
        <h1>{game ? games[game.config.game] : 'Room'}</h1>
        <span class="muted" role="status"
          >{connection?.status === 'connected' ? '' : 'Reconnecting…'}</span
        >
      </div>
      <svelte:element this={crewTable ? 'details' : 'div'} class="room-setup">
        {#if crewTable}<summary>Room · {code}</summary>{/if}
        <ul aria-label="Members">
          {#each room.members as member}
            <li>
              {member.nickname}{member.id === room.you
                ? ' · you'
                : ''}{member.id === room.host ? ' · host' : ''}
            </li>
          {/each}
        </ul>
        {#if room.config}<p>Selected game: {games[room.config.game]}</p>{/if}
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
                    connection?.seat(room.seating.filter((m) => m !== member))}
                  >Bench {room.members[member].nickname}</button
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
                  onclick={() => connection?.seat([...room.seating, member.id])}
                  >Seat {member.nickname}</button
                >{/if}
            </li>
          {/each}
        </ul>
      </svelte:element>
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
              <li class:active={active && game.turn.players.includes(index)}>
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
                onclick={() => connection?.finish(false)}>Finish: lost</button
              >
            </div>
          {/if}
        {/if}
        <GameBoard
          room={game}
          roomInfo={{ code: room.code, host: room.host }}
          disabled={connection?.pending || connection?.status !== 'connected'}
          move={(action) => connection?.move(action)}
        />
      {/if}
      {#if room.replays.length}<details class="replays">
          <summary>Replays ({room.replays.length})</summary
          >{#each room.replays as archive, index}<button
              class="quiet"
              onclick={() => replay(archive.id)}
              >Game {index + 1} · {games[archive.config.game]} ↓</button
            >{/each}
        </details>{/if}
    </section>
  {/if}
</main>

<style>
  main.crew-shell {
    max-width: 90rem;
    background: var(--arena-canvas);
    min-height: 100svh;
  }
  .crew-shell header {
    margin-bottom: 0.75rem;
    padding-right: 4rem;
  }
  .crew-shell .room-title h1 {
    margin-bottom: 0.5rem;
  }
  .crew-shell .room-setup {
    margin-bottom: 0.5rem;
  }
  .crew-shell .room-setup > :global(summary) {
    padding: 0.5rem 0;
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
    .crew-shell header {
      padding-right: 0;
    }
    main.entry-shell {
      padding: var(--arena-space-4) var(--arena-space-2) 6rem;
    }
  }
</style>
