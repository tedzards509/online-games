<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createRoom,
    downloadReplay,
    joinRoom,
    loadMembership,
    savedRooms,
    saveMembership,
  } from './lib/api';
  import { RoomConnection } from './lib/connection.svelte';
  import { games, type GameKind, type Membership } from './lib/types';
  import GameBoard from './components/GameBoard.svelte';
  import GameConfig from './components/GameConfig.svelte';
  import { crazyEightsDefaults, crazyEightsRules } from './lib/config';

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
  let kind = $state<GameKind>('shithead');
  let crazyEightsConfig = $state({ ...crazyEightsDefaults });
  let fourCardBombs = $state(false);
  let busy = $state(false);
  let error = $state('');
  let recent = $state(savedRooms());
  let copied = $state(false);
  const room = $derived(connection?.room);

  function navigate(next: string): void {
    history.pushState(null, '', next ? `/room/${next}` : '/');
    syncLocation();
  }
  function syncLocation(): void {
    code = pathCode();
    joinCode = code;
    member = code ? loadMembership(code) : null;
    error = '';
    recent = savedRooms();
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
    try {
      const target = joinCode.trim().toUpperCase();
      const existing = !create ? loadMembership(target) : null;
      const membership =
        existing ??
        (await (create
          ? createRoom(
              nickname,
              kind,
              kind === 'crazy_eights'
                ? { game: kind, rules: crazyEightsConfig }
                : kind === 'shithead'
                  ? { game: kind, rules: { four_card_bombs: fourCardBombs } }
                  : { game: kind, rules: {} },
            )
          : joinRoom(target, nickname)));
      saveMembership(membership);
      localStorage.setItem('arena-nickname', nickname.trim());
      navigate(membership.code);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not enter room';
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

<main>
  <header>
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
  </header>

  {#if error || connection?.error}<p class="error" role="alert">
      {error || connection?.error}
    </p>{/if}

  {#if !member}
    <section class="entry">
      <h1>{code ? 'Join room' : 'A table for friends.'}</h1>
      <label
        >Nickname<input
          bind:value={nickname}
          maxlength="24"
          autocomplete="nickname"
          required
        /></label
      >
      {#if !code}
        <form
          onsubmit={(event) => {
            event.preventDefault();
            enter(true);
          }}
        >
          <label
            >Game<select bind:value={kind}
              >{#each Object.entries(games) as [value, name]}<option {value}
                  >{name}</option
                >{/each}</select
            ></label
          >
          {#if kind === 'crazy_eights'}
            <fieldset>
              <legend>Game rules</legend>
              {#each Object.entries(crazyEightsRules) as [key, label]}
                <label class="rule-toggle">
                  <input
                    type="checkbox"
                    bind:checked={
                      crazyEightsConfig[key as keyof typeof crazyEightsConfig]
                    }
                  />
                  {label}
                </label>
              {/each}
            </fieldset>
          {:else if kind === 'shithead'}
            <label class="rule-toggle">
              <input type="checkbox" bind:checked={fourCardBombs} />
              Four-card bombs bypass rank restrictions
            </label>
          {/if}
          <button class="primary" disabled={busy || !nickname.trim()}
            >Create room</button
          >
        </form>
        <div class="divider"></div>
      {/if}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          enter(false);
        }}
      >
        <label
          >Room code<input
            class="code-input"
            bind:value={joinCode}
            maxlength="6"
            minlength="6"
            pattern={'[A-Za-z2-9]{6}'}
            autocapitalize="characters"
            spellcheck="false"
            required
          /></label
        >
        <button
          disabled={busy || !nickname.trim() || joinCode.trim().length !== 6}
          >Join room</button
        >
      </form>
      {#if recent.length && !code}<nav class="recent" aria-label="Your rooms">
          {#each recent as saved}<a
              href={`/room/${saved}`}
              onclick={(event) => {
                event.preventDefault();
                navigate(saved);
              }}>{saved}</a
            >{/each}
        </nav>{/if}
    </section>
  {:else if !room}
    <p role="status">
      {connection?.status === 'offline'
        ? 'Disconnected. Reconnecting…'
        : 'Connecting…'}
    </p>
  {:else}
    <section class="room" aria-label="Game room">
      <div class="room-title">
        <h1>{games[room.game]}</h1>
        <span class="muted" role="status"
          >{connection?.status === 'connected' ? '' : 'Reconnecting…'}</span
        >
      </div>
      <GameConfig config={room.config} />
      <ul class="players" aria-label="Players">
        {#each room.players as player, index}
          <li
            class:active={room.view &&
              !room.outcome &&
              room.turn?.players.includes(index)}
          >
            {player.nickname}{index === room.you ? ' · you' : ''}{index === 0
              ? ' · host'
              : ''}
          </li>
        {/each}
      </ul>
      {#if !room.view}
        {#if room.you === 0}<button
            class="primary"
            disabled={room.players.length < 2 ||
              connection?.pending ||
              connection?.status !== 'connected'}
            onclick={() => connection?.start()}>Start game</button
          >{:else}<p class="muted">Waiting for host</p>{/if}
      {:else}
        {#if room.outcome}
          <h2 role="status">
            {room.outcome.winners.length
              ? `${room.outcome.winners.map((p) => room.players[p].nickname).join(', ')} won`
              : 'Draw'}
          </h2>
          {#if room.outcome.scores}<p>
              {room.outcome.scores
                .map((score, p) => `${room.players[p].nickname}: ${score}`)
                .join(' · ')}
            </p>{/if}
          {#if room.you === 0}<button
              onclick={() => connection?.start()}
              disabled={connection?.pending ||
                connection?.status !== 'connected'}>Play again</button
            >{/if}
        {:else}
          <p class="turn" role="status">
            {room.turn?.players.includes(room.you)
              ? 'Your turn'
              : `${room.turn?.players.map((player) => room.players[player].nickname).join(', ') || 'Nobody'}'s turn`}
          </p>
        {/if}
        <GameBoard
          {room}
          disabled={connection?.pending || connection?.status !== 'connected'}
          move={(action) => connection?.move(action)}
        />
      {/if}
      {#if room.replays.length}<details class="replays">
          <summary>Replays ({room.replays.length})</summary
          >{#each room.replays as id, index}<button
              class="quiet"
              onclick={() => replay(id)}>Game {index + 1} ↓</button
            >{/each}
        </details>{/if}
    </section>
  {/if}
</main>

<style>
  fieldset {
    min-width: 0;
  }
  .rule-toggle {
    display: flex;
    align-items: baseline;
    margin-block: 8px;
  }
  .rule-toggle input {
    width: auto;
    flex-shrink: 0;
  }
</style>
